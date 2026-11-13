use histcite_core::{MatchingMode, analyze_files_with_mode};
fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut mode = MatchingMode::Histcite;
    if args.first().is_some_and(|s| s == "--mode") {
        mode = match args.get(1).map(String::as_str) {
            Some("histcite") => MatchingMode::Histcite,
            Some("doi") => MatchingMode::Doi,
            _ => {
                eprintln!("--mode 必須是 histcite 或 doi");
                std::process::exit(2)
            }
        };
        args.drain(..2);
    }
    if args.len() < 3 || args[0] != "--output" {
        eprintln!("用法：histcite-cli --output 結果.json 檔案1.txt [檔案2.txt ...]");
        std::process::exit(2);
    }
    let result = (|| -> Result<(), String> {
        let analysis = analyze_files_with_mode(&args[2..], mode)?;
        let output = serde_json::to_string_pretty(&analysis).map_err(|e| e.to_string())?;
        std::fs::write(&args[1], output).map_err(|e| e.to_string())?;
        eprintln!(
            "{} 篇文獻，{} 條參考文獻，{} 條本地引用連線，{} 條歧義。",
            analysis.records.len(),
            analysis.reference_count,
            analysis.edges.len(),
            analysis.ambiguities.len()
        );
        for warning in &analysis.warnings {
            eprintln!("提醒：{warning}");
        }
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
