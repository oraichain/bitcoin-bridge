FROM rust:1.73

RUN apt update && apt install build-essential libssl-dev pkg-config clang git nano -y
RUN rustup default nightly

WORKDIR /workspace

RUN git clone https://github.com/oraichain/bitcoin-bridge.git
WORKDIR /workspace/bitcoin-bridge
RUN git checkout develop
RUN cargo install --locked --path .