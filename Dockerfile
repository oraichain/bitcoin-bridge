FROM rust:1.73

RUN apt update && apt install build-essential libssl-dev pkg-config clang git nano -y
RUN rustup default nightly

WORKDIR /workspace

RUN apt install git -y
RUN git clone https://github.com/oraichain/bitcoin-bridge.git
RUN apt install byobu -y
RUN apt install jq -y
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
