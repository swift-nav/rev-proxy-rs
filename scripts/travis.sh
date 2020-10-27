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

if [[ "$TRAVIS_OS_NAME" == "windows" ]]; then
  cd target/release;
  7z a -tzip ../../revproxy_windows.zip rev-proxy.exe;
  cd ../..;
  VERSION="$(git describe --always --tags)";
  mv revproxy_windows.zip "revproxy-${VERSION}-windows-x86_64.zip";
  ls -l;
fi

if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
  tar -C "target/release" -czf revproxy_macos.tar.gz rev-proxy;
  VERSION="$(git describe --always --tags)";
  mv revproxy_macos.tar.gz "revproxy-${VERSION}-macos-x86_64.tar.gz";
  ls -l;
fi

if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then
  tar -C "target/x86_64-unknown-linux-musl/release" -czf revproxy_linux.tar.gz rev-proxy;
  VERSION="$(git describe --always --tags)";
  mv revproxy_linux.tar.gz "revproxy-${VERSION}-linux-x86_64.tar.gz";
  ls -l;
fi
