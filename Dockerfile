# The following line activates a syntax version for this Dockerfile that allows us to
# use heredoc.
# See
# - https://www.docker.com/blog/introduction-to-heredocs-in-dockerfiles/
# - https://github.com/moby/moby/issues/16058#issuecomment-881901519
# - https://hub.docker.com/r/docker/dockerfile.
# syntax=docker/dockerfile:1.3

FROM rust:1.63.0-alpine as builder

ARG BINARYEN_VERSION="version_105"
ARG SCCACHE_VERSION="0.2.15"
ARG SCCACHE_CHECKSUM_AMD64="e5d03a9aa3b9fac7e490391bbe22d4f42c840d31ef9eaf127a03101930cbb7ca"
ARG SCCACHE_CHECKSUM_ARM64="90d91d21a767e3f558196dbd52395f6475c08de5c4951a4c8049575fa6894489"
ARG TARGETARCH

# Download binaryen sources
ADD https://github.com/WebAssembly/binaryen/archive/refs/tags/$BINARYEN_VERSION.tar.gz /tmp/binaryen.tar.gz

# Extract and compile wasm-opt
# Adapted from https://github.com/WebAssembly/binaryen/blob/main/.github/workflows/build_release.yml
RUN apk update && apk add build-base cmake git python3 clang ninja
RUN tar -xf /tmp/binaryen.tar.gz
RUN cd binaryen-version_*/ && cmake . -G Ninja -DCMAKE_CXX_FLAGS="-static" -DCMAKE_C_FLAGS="-static" -DCMAKE_BUILD_TYPE=Release -DBUILD_STATIC_LIB=ON && ninja wasm-opt

# Run tests
RUN cd binaryen-version_*/ && ninja wasm-as wasm-dis
RUN cd binaryen-version_*/ && python3 check.py wasm-opt

# Install wasm-opt
RUN strip binaryen-version_*/bin/wasm-opt
RUN mv binaryen-version_*/bin/wasm-opt /usr/local/bin

# Check cargo version
RUN cargo --version

# Check wasm-opt version
RUN wasm-opt --version

# Install sccache
RUN <<EOT ash
  if [ "amd64" = "$TARGETARCH" ]; then
    wget -O /tmp/sccache.tar.gz https://github.com/mozilla/sccache/releases/download/v$SCCACHE_VERSION/sccache-v$SCCACHE_VERSION-x86_64-unknown-linux-musl.tar.gz
    sha256sum /tmp/sccache.tar.gz | grep "$SCCACHE_CHECKSUM_AMD64"
  elif [ "arm64" = "$TARGETARCH" ]; then
    wget -O /tmp/sccache.tar.gz https://github.com/mozilla/sccache/releases/download/v$SCCACHE_VERSION/sccache-v$SCCACHE_VERSION-aarch64-unknown-linux-musl.tar.gz
    sha256sum /tmp/sccache.tar.gz | grep "$SCCACHE_CHECKSUM_ARM64"
  else
    echo "Got unexpected value for TARGETARCH: '$TARGETARCH'"
    exit 42
  fi

  tar -xf /tmp/sccache.tar.gz
  mv sccache-v*/sccache /usr/local/bin/sccache
  chmod +x /usr/local/bin/sccache
  rm /tmp/sccache.tar.gz

  sccache --version
EOT

# Add scripts
ADD optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

ADD optimize_workspace.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/optimize_workspace.sh

# Being required for gcc linking of build_workspace
RUN apk add --no-cache musl-dev

ADD build_workspace build_workspace
RUN cd build_workspace && \
  echo "Installed targets:" && (rustup target list | grep installed) && \
  export DEFAULT_TARGET="$(rustc -vV | grep 'host:' | cut -d' ' -f2)" && echo "Default target: $DEFAULT_TARGET" && \
  # Those RUSTFLAGS reduce binary size from 4MB to 600 KB
  RUSTFLAGS='-C link-arg=-s' cargo build --release && \
  ls -lh target/release/build_workspace && \
  (ldd target/release/build_workspace || true) && \
  mv target/release/build_workspace /usr/local/bin

#
# base-optimizer
#
FROM rust:1.63.0-alpine as base-optimizer

# Being required for gcc linking
RUN apk update && \
  apk add --no-cache musl-dev

# Setup Rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

# Add wasm-opt
COPY --from=builder /usr/local/bin/wasm-opt /usr/local/bin

#
# rust-optimizer
#
FROM base-optimizer as rust-optimizer

# Use sccache. Users can override this variable to disable caching.
COPY --from=builder /usr/local/bin/sccache /usr/local/bin
ENV RUSTC_WRAPPER=sccache

# Assume we mount the source code in /code
WORKDIR /code

# Add script as entry point
COPY --from=builder /usr/local/bin/optimize.sh /usr/local/bin

ENTRYPOINT ["optimize.sh"]
# Default argument when none is provided
CMD ["."]

#
# workspace-optimizer
#
FROM base-optimizer as workspace-optimizer

# Assume we mount the source code in /code
WORKDIR /code

# Add script as entry point
COPY --from=builder /usr/local/bin/optimize_workspace.sh /usr/local/bin
COPY --from=builder /usr/local/bin/build_workspace /usr/local/bin

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
