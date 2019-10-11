# Usage
# docker build . -t confio/cosmwasm-opt:1.38
# docker run --rm -v PATH/TO/CONTRACT/Cargo.toml:/code confio/cosmwasm-opt:latest
# docker run --rm -v "$(pwd)":/code confio/cosmwasm-opt:latest
# docker run --rm -it -v "$(pwd)":/code confio/cosmwasm-opt:latest /bin/bash
FROM trzeci/emscripten:1.38.47

#ldd /emsdk_portable/binaryen/bin/wasm-opt
#        linux-vdso.so.1 (0x00007ffcd2923000)
#        libstdc++.so.6 => /usr/lib/x86_64-linux-gnu/libstdc++.so.6 (0x00007fb985130000)
#        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007fb984e2c000)
#        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007fb984c15000)
#        libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007fb9849f8000)
#        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007fb984659000)
#        /lib64/ld-linux-x86-64.so.2 (0x00007fb985cea000)


FROM rust:1.38.0

# copy all binaryen links
COPY --from=0 /emsdk_portable/binaryen /emsdk_portable/binaryen

# setup rust with wasm-pack
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack --version 0.8.1

# Assumes we mount the source code in /code
WORKDIR /code
ADD optimize.sh /root/optimize.sh
RUN chmod +x /root/optimize.sh

## TODO: we need to .cargo/bin in PATH
## Also /emsdk_portable:/emsdk_portable/clang/tag-e1.38.47/build_tag-e1.38.47_64/bin:/emsdk_portable/node/8.9.1_64bit/bin:/emsdk_portable/emscripten/tag-1.38.47:/emsdk_portable/binaryen/tag-1.38.47_64bit_binaryen/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
## Maybe only /emsdk_portable/binaryen/bin

CMD ["/root/optimize.sh"]