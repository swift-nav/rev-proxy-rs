#!/usr/bin/env bash

set -ex

rustup component add clippy
rustup component add fmt

RUSTFLAGS="--deny=warnings" \
    cargo check --all-targets --all-features

cargo clippy --all-targets -- --deny=warnings
cargo fmt -- --check

if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then

  docker build --tag musl-builder .
  docker run -it --name musl-builder-run musl-builder

  mkdir -p target/x86_64-unknown-linux-musl/release

  docker cp musl-builder-run:/home/rust/src/target/x86_64-unknown-linux-musl/release/rev-proxy \
    target/x86_64-unknown-linux-musl/release/rev-proxy

  docker rm musl-builder-run
  docker rmi musl-builder

elif [[ "$TRAVIS_OS_NAME" == "osx" ]] || [[ "$TRAVIS_OS_NAME" == "windows" ]]; then

  cargo build --release
fi
