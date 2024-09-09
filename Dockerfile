FROM rust:1.81.0-alpine AS targetarch

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

ARG BINARYEN_VERSION="version_116"

RUN echo "Running on $BUILDPLATFORM, building for $TARGETPLATFORM"

# AMD64
FROM targetarch AS builder-amd64
ARG ARCH="x86_64"

# ARM64
FROM targetarch AS builder-arm64
ARG ARCH="aarch64"

# GENERIC
# The builder image builds binaries like wasm-opt and bob.
# After the build process, only the final binaries are copied into the *-optimizer
# images to avoid shipping all the source code and intermediate build results to the user.
FROM builder-${TARGETARCH} AS builder

# Download binaryen sources
ADD https://github.com/WebAssembly/binaryen/archive/refs/tags/$BINARYEN_VERSION.tar.gz /tmp/binaryen.tar.gz

# Extract and compile wasm-opt
# Adapted from https://github.com/WebAssembly/binaryen/blob/main/.github/workflows/build_release.yml
RUN apk update && apk add build-base cmake git python3 clang ninja
RUN tar -xf /tmp/binaryen.tar.gz
RUN cd binaryen-version_*/ \
  && git clone --depth 1 https://github.com/google/googletest.git ./third_party/googletest \
  && cmake . -G Ninja -DCMAKE_CXX_FLAGS="-static" -DCMAKE_C_FLAGS="-static" -DCMAKE_BUILD_TYPE=Release -DBUILD_STATIC_LIB=ON \
  && ninja wasm-opt

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

# Add scripts
ADD optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

# Being required for gcc linking of bob
RUN apk add --no-cache musl-dev

# Copy crate source
ADD bob_the_builder bob_the_builder

# Download the crates.io index using the new sparse protocol to improve performance
# and avoid OOM in the bob_the_builder build.
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Build bob binary
# Those RUSTFLAGS reduce binary size from 4MB to 600 KB
RUN cd bob_the_builder && RUSTFLAGS='-C link-arg=-s' cargo build --release
# Check bob binary
RUN cd bob_the_builder && \
  ls -lh target/release/bob && \
  (ldd target/release/bob || true) && \
  mv target/release/bob /usr/local/bin

#
# rust-optimizer target
#
FROM rust:1.81.0-alpine AS rust-optimizer

# Download the crates.io index using the new sparse protocol to improve performance
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Being required for gcc linking
RUN apk update && \
  apk add --no-cache musl-dev

# Setup Rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

# Add bob and wasm-opt
COPY --from=builder /usr/local/bin/bob /usr/local/bin
COPY --from=builder /usr/local/bin/wasm-opt /usr/local/bin

# Add script as entry point
COPY --from=builder /usr/local/bin/optimize.sh /usr/local/bin

# Assume we mount the source code in /code
WORKDIR /code

ENTRYPOINT ["optimize.sh"]
# Default argument when none is provided
CMD ["."]
