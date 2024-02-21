#!/bin/bash

# stop on error & display the running command
set -xe

# only required features
cargo test --verbose --no-default-features --features=full,feat-ibc,testnet

# all features
cargo test --verbose --features full,feat-ibc,testnet,faucet-test,devnet

# check rest
cargo check --manifest-path rest/Cargo.toml --verbose

# run code coverage
cargo llvm-cov --no-default-features --features=full,feat-ibc,testnet --workspace --lcov --no-cfg-coverage-nightly

# formatter
cargo fmt --all -- --check

# clippy
cargo clippy --no-default-features --features=full,feat-ibc,testnet -- -D warnings

# test bitcoin
cargo test --verbose --features full,feat-ibc,testnet,faucet-test,devnet bitcoin -- --ignored