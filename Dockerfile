# See https://github.com/emscripten-core/emscripten/pull/9119/files for fastcomp vs. upstream docs
# Using the traditional fastcomp for now.
FROM trzeci/emscripten:1.39.8-fastcomp

# Note: I tried slim and had issues compiling wasm-pack, even with --features vendored-openssl
FROM rust:1.41.0

# setup rust with wasm-pack
RUN rustup target add wasm32-unknown-unknown

# I'd rather not
# RUN cargo install wasm-pack --version 0.9.1
RUN curl -L -sSf https://github.com/rustwasm/wasm-pack/releases/download/v0.9.1/wasm-pack-v0.9.1-x86_64-unknown-linux-musl.tar.gz > wasm-pack.tar.gz
RUN tar xzf wasm-pack.tar.gz
RUN cp wasm-pack-*-x86_64-unknown-linux-musl/wasm-pack /usr/local/bin

# cleanup
RUN rm -rf wasm-*

# copy wasm-opt into our path
COPY --from=0 /emsdk_portable/binaryen/bin/wasm-opt /usr/local/bin

# Assume we mount the source code in /code
WORKDIR /code

# Add out script as entry point
ADD optimize.sh /opt/optimize.sh
RUN chmod +x /opt/optimize.sh

ENTRYPOINT ["/opt/optimize.sh"]
# Default argument when none is provided
CMD ["."]
