FROM rust:1.54.0-alpine as targetarch

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

ARG BINARYEN_VERSION="version_101"
ARG BINARYEN_CHECKSUM="20d0b19ca716c51d927f181802125f04d5685250c8a22ec3022ac28bf4f20c57"

RUN echo "Running on $BUILDPLATFORM, building for $TARGETPLATFORM"

# AMD64
FROM targetarch as builder-amd64
ARG ARCH="x86_64"


# Download binaryen binary and verify checksum
ADD https://github.com/WebAssembly/binaryen/releases/download/$BINARYEN_VERSION/binaryen-$BINARYEN_VERSION-x86_64-linux.tar.gz /tmp/binaryen.tar.gz
RUN sha256sum /tmp/binaryen.tar.gz | grep $BINARYEN_CHECKSUM

# Extract wasm-opt
RUN tar -xf /tmp/binaryen.tar.gz

# ARM64
FROM targetarch as builder-arm64
ARG ARCH="aarch64"

# Download binaryen sources
ADD https://github.com/WebAssembly/binaryen/archive/refs/tags/$BINARYEN_VERSION.tar.gz /tmp/binaryen.tar.gz

# Extract and compile wasm-opt
# Adapted from https://github.com/WebAssembly/binaryen/blob/main/.github/workflows/build_release.yml
RUN apk update && apk add build-base cmake git python3 clang ninja
RUN tar -xf /tmp/binaryen.tar.gz
RUN cd binaryen-version_*/ && cmake . -G Ninja -DCMAKE_CXX_FLAGS="-static" -DCMAKE_C_FLAGS="-static" -DCMAKE_BUILD_TYPE=Release -DBUILD_STATIC_LIB=ON && ninja wasm-opt
RUN strip binaryen-version_*/bin/wasm-opt

# GENERIC
FROM builder-${TARGETARCH} as builder

# Install wasm-opt
RUN mv binaryen-version_*/bin/wasm-opt /usr/local/bin

# Check cargo version
RUN cargo --version

# Check wasm-opt version
RUN wasm-opt --version

# Download sccache and verify checksum
ADD https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-$ARCH-unknown-linux-musl.tar.gz /tmp/sccache.tar.gz
RUN sha256sum /tmp/sccache.tar.gz | egrep '(e5d03a9aa3b9fac7e490391bbe22d4f42c840d31ef9eaf127a03101930cbb7ca|90d91d21a767e3f558196dbd52395f6475c08de5c4951a4c8049575fa6894489)'

# Extract and install sccache
RUN tar -xf /tmp/sccache.tar.gz
RUN mv sccache-v*/sccache /usr/local/bin/sccache
RUN chmod +x /usr/local/bin/sccache

# Check sccache version
RUN sccache --version

# Add scripts
ADD optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

ADD optimize_workspace.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/optimize_workspace.sh
ADD build_workspace.py /usr/local/bin/
RUN chmod +x /usr/local/bin/build_workspace.py

#
# base-optimizer
#
FROM rust:1.54.0-alpine as base-optimizer

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

# Install Python
RUN apk add python3 py3-toml
RUN python3 --version

# Assume we mount the source code in /code
WORKDIR /code

# Add script as entry point
COPY --from=builder /usr/local/bin/optimize_workspace.sh /usr/local/bin
COPY --from=builder /usr/local/bin/build_workspace.py /usr/local/bin

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
