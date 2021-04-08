# SHARED BLOCK START (keep in sync between both *.Dockerfile to utilize Docker caching)
FROM rust:1.51.0-alpine

RUN apk update
# Being required for gcc linking
RUN apk add --no-cache musl-dev

# Setup Rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

# Check cargo version
RUN cargo --version

# Download binaryen and verify checksum
ADD https://github.com/WebAssembly/binaryen/releases/download/version_96/binaryen-version_96-x86_64-linux.tar.gz /tmp/binaryen.tar.gz
RUN sha256sum /tmp/binaryen.tar.gz | grep 9f8397a12931df577b244a27c293d7c976bc7e980a12457839f46f8202935aac

# Extract and install wasm-opt
RUN tar -xf /tmp/binaryen.tar.gz
RUN mv binaryen-version_*/wasm-opt /usr/local/bin
RUN rm -rf binaryen-version_*/ /tmp/binaryen.tar.gz

# Check wasm-opt version
RUN wasm-opt --version
# SHARED BLOCK END

# Download sccache and verify checksum
ADD https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-x86_64-unknown-linux-musl.tar.gz /tmp/sccache.tar.gz
RUN sha256sum /tmp/sccache.tar.gz | grep e5d03a9aa3b9fac7e490391bbe22d4f42c840d31ef9eaf127a03101930cbb7ca

# Extract and install sccache
RUN tar -xf /tmp/sccache.tar.gz
RUN mv sccache-v*/sccache /usr/local/bin/sccache
RUN chmod +x /usr/local/bin/sccache
RUN rm -rf sccache-v*/ /tmp/sccache.tar.gz

# Check sccache version
RUN sccache --version

# Use sccache. Users can override this variable to disable caching.
ENV RUSTC_WRAPPER=sccache

# Assume we mount the source code in /code
WORKDIR /code

# Add out script as entry point
ADD optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

ENTRYPOINT ["optimize.sh"]
# Default argument when none is provided
CMD ["."]
