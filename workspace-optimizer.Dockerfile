FROM rust:1.51.0-alpine

# Setup Rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

RUN apk update
# Being required for gcc linking
RUN apk add --no-cache musl-dev

RUN apk add python3 py3-toml

RUN python3 --version

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

# Assume we mount the source code in /code
WORKDIR /code

# Add our scripts as entry point
ADD build_workspace.py /usr/local/bin/
ADD optimize_workspace.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/build_workspace.py
RUN chmod +x /usr/local/bin/optimize_workspace.sh

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
