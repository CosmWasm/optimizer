FROM rust:1.54.0-alpine as builder

# Check cargo version
RUN cargo --version

# Download binaryen and verify checksum
ADD https://github.com/WebAssembly/binaryen/releases/download/version_96/binaryen-version_96-x86_64-linux.tar.gz /tmp/binaryen.tar.gz
RUN sha256sum /tmp/binaryen.tar.gz | grep 9f8397a12931df577b244a27c293d7c976bc7e980a12457839f46f8202935aac

# Extract and install wasm-opt
RUN tar -xf /tmp/binaryen.tar.gz
RUN mv binaryen-version_*/wasm-opt /usr/local/bin

# Check wasm-opt version
RUN wasm-opt --version

# Download sccache and verify checksum
ADD https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-x86_64-unknown-linux-musl.tar.gz /tmp/sccache.tar.gz
RUN sha256sum /tmp/sccache.tar.gz | grep e5d03a9aa3b9fac7e490391bbe22d4f42c840d31ef9eaf127a03101930cbb7ca

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