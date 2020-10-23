# Note: I tried slim and had issues compiling wasm-pack, even with --features vendored-openssl
FROM rust:1.47.0

# setup rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

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

# Add out script as entry point
ADD optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

ENTRYPOINT ["optimize.sh"]
# Default argument when none is provided
CMD ["."]
