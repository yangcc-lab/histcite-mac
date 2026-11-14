//! Serialize workspace work without blocking the async executor or the UI thread.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Semaphore;

const MAX_REQUESTS: usize = 32;

pub struct Background<T> {
    value: Arc<Mutex<T>>,
    execution: Arc<Semaphore>,
    admission: Arc<Semaphore>,
    closing: AtomicBool,
}

impl<T: Default> Default for Background<T> {
    fn default() -> Self {
        Self {
            value: Arc::new(Mutex::new(T::default())),
            execution: Arc::new(Semaphore::new(1)),
            admission: Arc::new(Semaphore::new(MAX_REQUESTS)),
            closing: AtomicBool::new(false),
        }
    }
}

impl<T: Send + 'static> Background<T> {
    pub async fn finish(&self) {
        // Refuse new work, then wait behind jobs already admitted. Native quit
        // must not terminate the process in the middle of a save/recovery write.
        self.closing.store(true, Ordering::SeqCst);
        // Drain admission, not just execution: an admitted caller can be
        // preempted before it joins the execution queue. This private semaphore
        // is never closed, so acquiring all permits waits for every such caller.
        let _permits = self.admission.acquire_many(MAX_REQUESTS as u32).await;
    }

    pub async fn run<R, F>(&self, work: F) -> Result<R, String>
    where
        R: Send + 'static,
        F: FnOnce(&mut T) -> Result<R, String> + Send + 'static,
    {
        if self.closing.load(Ordering::SeqCst) {
            return Err("App 正在結束，無法接受新的操作。".into());
        }
        let admission = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| "目前有太多待處理操作，請稍候再試。".to_string())?;
        // A caller that raced the shutdown gate must not start after the drain.
        if self.closing.load(Ordering::SeqCst) {
            return Err("App 正在結束，無法接受新的操作。".into());
        }
        let execution = self
            .execution
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;
        let value = self.value.clone();
        tokio::task::spawn_blocking(move || {
            // Keep both permits inside the job: dropping an IPC future does not
            // cancel an already-running write or allow a second job to overlap it.
            let (_admission, _execution) = (admission, execution);
            let mut state = value.lock().map_err(|e| e.to_string())?;
            work(&mut state)
        })
        .await
        .map_err(|e| format!("背景操作失敗：{e}"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn synchronous_work_leaves_async_executor_available() {
        runtime().block_on(async {
            let state = Arc::new(Background::<usize>::default());
            let (started, ready) = tokio::sync::oneshot::channel();
            let (release, wait) = mpsc::channel();
            let task = tokio::spawn(async move {
                state
                    .run(move |value| {
                        started.send(()).unwrap();
                        wait.recv_timeout(Duration::from_secs(5))
                            .map_err(|e| e.to_string())?;
                        *value += 1;
                        Ok(*value)
                    })
                    .await
            });
            ready.await.unwrap();
            // This can run on a single-thread executor while synchronous work waits.
            release.send(()).unwrap();
            assert_eq!(task.await.unwrap().unwrap(), 1);
        });
    }

    #[test]
    fn cancelled_caller_keeps_running_job_exclusive() {
        runtime().block_on(async {
            let state = Arc::new(Background::<usize>::default());
            let worker = state.clone();
            let (started, ready) = tokio::sync::oneshot::channel();
            let (release, wait) = mpsc::channel();
            let task = tokio::spawn(async move {
                worker
                    .run(move |value| {
                        started.send(()).unwrap();
                        wait.recv_timeout(Duration::from_secs(5))
                            .map_err(|e| e.to_string())?;
                        *value = 42;
                        Ok(())
                    })
                    .await
            });
            ready.await.unwrap();
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
            assert_eq!(state.execution.available_permits(), 0);
            assert_eq!(state.admission.available_permits(), MAX_REQUESTS - 1);
            release.send(()).unwrap();
            assert_eq!(state.run(|value| Ok(*value)).await.unwrap(), 42);
            assert_eq!(state.admission.available_permits(), MAX_REQUESTS);
        });
    }

    #[test]
    fn revisions_are_checked_after_queued_work() {
        runtime().block_on(async {
            let state = Arc::new(Background::<usize>::default());
            let mut jobs = Vec::new();
            for _ in 0..2 {
                let worker = state.clone();
                jobs.push(tokio::spawn(async move {
                    worker
                        .run(|revision| {
                            if *revision != 0 {
                                return Err("stale revision".into());
                            }
                            *revision += 1;
                            Ok(())
                        })
                        .await
                }));
            }
            let mut successful = 0;
            for job in jobs {
                successful += usize::from(job.await.unwrap().is_ok());
            }
            assert_eq!(successful, 1);
            assert_eq!(state.run(|revision| Ok(*revision)).await.unwrap(), 1);
        });
    }

    #[test]
    fn full_queue_rejects_without_running_or_mutating() {
        runtime().block_on(async {
            let state = Background::<usize>::default();
            let slots = state
                .admission
                .acquire_many(MAX_REQUESTS as u32)
                .await
                .unwrap();
            assert!(
                state
                    .run(|value| {
                        *value = 99;
                        Ok(())
                    })
                    .await
                    .is_err()
            );
            drop(slots);
            assert_eq!(state.run(|value| Ok(*value)).await.unwrap(), 0);
        });
    }

    #[test]
    fn shutdown_waits_for_admitted_caller_before_execution_queue() {
        runtime().block_on(async {
            let state = Arc::new(Background::<usize>::default());
            // Model a caller preempted between admission and execution enqueue.
            let caller = state.admission.clone().try_acquire_owned().unwrap();
            let exiting = state.clone();
            let shutdown = tokio::spawn(async move { exiting.finish().await });
            tokio::task::yield_now().await;
            assert!(!shutdown.is_finished());
            assert!(state.run(|_| Ok(())).await.is_err());
            drop(caller);
            shutdown.await.unwrap();
        });
    }

    #[test]
    fn shutdown_drains_running_write_and_refuses_new_work() {
        runtime().block_on(async {
            let state = Arc::new(Background::<usize>::default());
            let worker = state.clone();
            let (started, ready) = tokio::sync::oneshot::channel();
            let (release, wait) = mpsc::channel();
            let task = tokio::spawn(async move {
                worker
                    .run(move |value| {
                        started.send(()).unwrap();
                        wait.recv_timeout(Duration::from_secs(5))
                            .map_err(|e| e.to_string())?;
                        *value = 42;
                        Ok(())
                    })
                    .await
            });
            ready.await.unwrap();
            let queued = state.clone();
            let next = tokio::spawn(async move {
                queued
                    .run(|value| {
                        *value += 1;
                        Ok(())
                    })
                    .await
            });
            tokio::task::yield_now().await;
            let exiting = state.clone();
            let shutdown = tokio::spawn(async move { exiting.finish().await });
            tokio::task::yield_now().await;
            assert!(!shutdown.is_finished());
            assert!(state.run(|_| Ok(())).await.is_err());
            release.send(()).unwrap();
            task.await.unwrap().unwrap();
            next.await.unwrap().unwrap();
            shutdown.await.unwrap();
            assert_eq!(*state.value.lock().unwrap(), 43);
        });
    }
}
