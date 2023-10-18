FROM rust:1.72.0-alpine as targetarch

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

ARG BINARYEN_VERSION="version_114"

RUN echo "Running on $BUILDPLATFORM, building for $TARGETPLATFORM"

# AMD64
FROM targetarch as builder-amd64
ARG ARCH="x86_64"

# ARM64
FROM targetarch as builder-arm64
ARG ARCH="aarch64"

# GENERIC
# The builder image builds binaries like wasm-opt, sccache and bob.
# After the build process, only the final binaries are copied into the *-optimizer
# images to avoid shipping all the source code and intermediate build results to the user.
FROM builder-${TARGETARCH} as builder

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

# Download sccache and verify checksum
ADD https://github.com/mozilla/sccache/releases/download/v0.5.4/sccache-v0.5.4-$ARCH-unknown-linux-musl.tar.gz /tmp/sccache.tar.gz
RUN sha256sum /tmp/sccache.tar.gz | egrep '(4bf3ce366aa02599019093584a5cbad4df783f8d6e3610548c2044daa595d40b|85f0cfe9b3150e461801cf7453763080fc3604e255587e2a4886e55bb93e6b09)'

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
# base-optimizer
#
FROM rust:1.72.0-alpine as base-optimizer

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

# Use sccache. Users can override this variable to disable caching.
COPY --from=builder /usr/local/bin/sccache /usr/local/bin
ENV RUSTC_WRAPPER=sccache

#
# rust-optimizer
#
FROM base-optimizer as rust-optimizer

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

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
