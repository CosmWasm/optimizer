# Usage
FROM trzeci/emscripten:1.38.47

FROM rust:1.38.0

# setup rust with wasm-pack
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack --version 0.8.1

# copy wasm-opt into our path
COPY --from=0 /emsdk_portable/binaryen/bin/wasm-opt /usr/local/bin

# Assume we mount the source code in /code
WORKDIR /code

# Add out script as entry point
ADD optimize.sh /root/optimize.sh
RUN chmod +x /root/optimize.sh

CMD ["/root/optimize.sh"]