# See https://github.com/emscripten-core/emscripten/pull/9119/files for fastcomp vs. upstream docs
# Using the traditional fastcomp for now.
FROM trzeci/emscripten:1.39.8-fastcomp

# This version of Rust will not be used for compilation but just serves as a stable base image to get debian+rustup.
# See Rust nightly config below.
FROM rust:1.45.2
RUN rustup toolchain remove 1.45.2

RUN apt update
RUN apt install python3 python3-toml -y

RUN python3 --version

# Install Rust nightly
# Choose version from: https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu.html
RUN rustup toolchain install nightly-2020-08-20 --allow-downgrade --profile minimal --target wasm32-unknown-unknown
RUN rustup default nightly-2020-08-20
RUN rustup toolchain list
# Check cargo version
RUN cargo --version

# copy wasm-opt into our path
COPY --from=0 /emsdk_portable/binaryen/bin/wasm-opt /usr/local/bin

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
