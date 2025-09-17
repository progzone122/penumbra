FROM rust:latest

RUN apt-get update && apt-get install -y \
    musl-tools \
    gcc-mingw-w64-x86-64 \
    mingw-w64 \
    libudev-dev \
    pkg-config

RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /usr/src/penumbra-tui

CMD ["/bin/bash"]
