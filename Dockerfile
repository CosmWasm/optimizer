# See https://github.com/emscripten-core/emscripten/pull/9119/files for fastcomp vs. upstream docs
# Using the traditional fastcomp for now.
FROM trzeci/emscripten:1.39.8-fastcomp

# Note: I tried slim and had issues compiling wasm-pack, even with --features vendored-openssl
FROM rust:1.43.1

# setup rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

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
