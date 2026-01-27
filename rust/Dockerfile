FROM ubuntu:25.10

RUN apt-get update && apt-get install -y curl build-essential
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y -c rust-src,rustc-dev,llvm-tools-preview
