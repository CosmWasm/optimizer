FROM rust:1.54.0-alpine as targetarch

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

RUN echo "Running on $BUILDPLATFORM, building for $TARGETPLATFORM"

FROM targetarch as builder-amd64
ARG ARCH="x86_64"

FROM targetarch as builder-arm64
ARG ARCH="aarch64"

FROM builder-${TARGETARCH} as builder
# Check cargo version
RUN cargo --version

# Install platform-specific binaryen
RUN apk update && apk add binaryen

# Check wasm-opt version
RUN wasm-opt --version

# Download sccache and verify checksum
ADD https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-$ARCH-unknown-linux-musl.tar.gz /tmp/sccache.tar.gz
RUN sha256sum /tmp/sccache.tar.gz | egrep '(e5d03a9aa3b9fac7e490391bbe22d4f42c840d31ef9eaf127a03101930cbb7ca|90d91d21a767e3f558196dbd52395f6475c08de5c4951a4c8049575fa6894489)'

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

# Install platform-specific binaryen
RUN apk add binaryen

# Setup Rust with Wasm support
RUN rustup target add wasm32-unknown-unknown

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
