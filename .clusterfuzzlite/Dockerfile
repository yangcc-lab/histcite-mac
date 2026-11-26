FROM gcr.io/oss-fuzz-base/base-builder-rust@sha256:8c2ccb74bb4b63c10c47e8f91c6633181fa00ca4cc4a4751dd390443e6cc76aa

# The base image's bundled nightly predates the minimum Rust version for HistCite Mac.
RUN rustup toolchain install nightly-2026-11-25 --profile minimal
ENV RUSTUP_TOOLCHAIN=nightly-2026-11-25

WORKDIR $SRC/histcite-mac
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY fuzz/ fuzz/
COPY examples/demo-wos.txt examples/demo-wos.txt
COPY --chmod=0755 .clusterfuzzlite/build.sh $SRC/build.sh
