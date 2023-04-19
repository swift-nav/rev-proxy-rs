#!/usr/bin/env bash

set -e

echo "installing system dependencies for building openssl"

apt install -y linux-headers-generic

export OPENSSL_VERSION=1.1.1g

tag=$(dd if=/dev/urandom count=8 bs=1 status=none| md5sum | cut -f1 -d' ')

echo "building OpenSSL with musl toolchain"

curl -sSL https://musl.cc/aarch64-linux-musl-cross.tgz | \
  tar -C /opt -xvzf -

mkdir -p /usr/local/musl-aarch64/include/openssl && \
  ln -sf /usr/include/linux /usr/local/musl-aarch64/include/linux && \
  ln -sf /usr/include/aarch64-linux-gnu/asm /usr/local/musl-aarch64/include/asm && \
  ln -sf /usr/include/asm-generic /usr/local/musl-aarch64/include/asm-generic && \
  mkdir -p /tmp/rust-openssl-build-$tag && \
  cd /tmp/rust-openssl-build-$tag && \
  short_version="$(echo "$OPENSSL_VERSION" | sed s'/[a-z]$//' )" && \
  { curl -v -fLO "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" || \
    curl -v -fLO "https://www.openssl.org/source/old/$short_version/openssl-$OPENSSL_VERSION.tar.gz"; } && \
  tar xvzf "openssl-$OPENSSL_VERSION.tar.gz" && cd "openssl-$OPENSSL_VERSION" && \
  env CC=/opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc ./Configure no-shared no-zlib -fPIC --prefix=/usr/local/musl-aarch64 -DOPENSSL_NO_SECURE_MEMORY linux-aarch64 && \
  env C_INCLUDE_PATH=/usr/local/musl-aarch64/include/ make depend && \
  env C_INCLUDE_PATH=/usr/local/musl-aarch64/include/ make -j`nproc` && \
  make install_sw && \
  rm -v /usr/local/musl-aarch64/include/linux /usr/local/musl-aarch64/include/asm /usr/local/musl-aarch64/include/asm-generic && \
  rm -rf /tmp/rust-openssl-build-*
