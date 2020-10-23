# This version of Rust will not be used for compilation but just serves as a stable base image to get debian+rustup.
# See Rust nightly config below.
FROM rust:1.47.0
RUN rustup toolchain remove 1.47.0

RUN apt update
RUN apt install python3 python3-toml -y

RUN python3 --version

# Install Rust nightly
# Choose version from: https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu.html
RUN rustup toolchain install nightly-2020-10-14 --allow-downgrade --profile minimal --target wasm32-unknown-unknown
RUN rustup default nightly-2020-10-14
RUN rustup toolchain list
# Check cargo version
RUN cargo --version

# Download binaryen and verify checksum
ADD https://github.com/WebAssembly/binaryen/releases/download/version_90/binaryen-version_90-x86_64-linux.tar.gz /tmp/binaryen.tar.gz
RUN sha256sum /tmp/binaryen.tar.gz | grep ea0bf4151103b19fce5a184044b7492715078187e88fd95b997089a4a16af082

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
