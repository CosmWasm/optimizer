# This version of Rust will not be used for compilation but just serves as a stable base image to get debian+rustup.
# See Rust nightly config below.
FROM rust:1.51.0
RUN rustup toolchain remove 1.51.0

RUN apt update
RUN apt install python3 python3-toml -y

RUN python3 --version

# Install Rust nightly
# Choose version from: https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu.html
RUN rustup toolchain install nightly-2021-03-01 --allow-downgrade --profile minimal --target wasm32-unknown-unknown
RUN rustup default nightly-2021-03-01
RUN rustup toolchain list
# Check cargo version
RUN cargo --version

# Download binaryen and verify checksum
ADD https://github.com/WebAssembly/binaryen/releases/download/version_96/binaryen-version_96-x86_64-linux.tar.gz /tmp/binaryen.tar.gz
RUN sha256sum /tmp/binaryen.tar.gz | grep 9f8397a12931df577b244a27c293d7c976bc7e980a12457839f46f8202935aac

# Extract and install wasm-opt
RUN tar -xf /tmp/binaryen.tar.gz --wildcards '*/wasm-opt'
RUN mv binaryen-version_*/wasm-opt /usr/local/bin

# Check wasm-opt version
RUN wasm-opt --version

# Assume we mount the source code in /code
WORKDIR /code

# Add our scripts as entry point
ADD optimize_workspace.py /usr/local/bin/
ADD optimize_workspace.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/optimize_workspace.py
RUN chmod +x /usr/local/bin/optimize_workspace.sh

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
