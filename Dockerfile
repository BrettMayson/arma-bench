FROM rust:1 as rust-builder

WORKDIR /app

COPY ./client /app/client
COPY ./server /app/server
COPY ./extension /app/extension
COPY ./Cargo.toml /app/Cargo.toml

RUN cd /app && \
    cargo build -p arma-bench-server --release

FROM debian:bookworm-slim

LABEL maintainer="Brett - github.com/brettmayson"
LABEL org.opencontainers.image.source=https://github.com/brettmayson/arma-bench

RUN apt-get update \
    && \
    apt-get install -y --no-install-recommends --no-install-suggests \
        lib32stdc++6 \
        lib32gcc-s1 \
        libcurl4 \
        wget \
        ca-certificates \
        unzip \
    && \
    apt-get remove --purge -y \
    && \
    apt-get clean autoclean \
    && \
    apt-get autoremove -y \
    && \
    rm -rf /var/lib/apt/lists/* \
    && \
    mkdir -p /steamcmd \
    && \
    wget -qO- 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz' | tar zxf - -C /steamcmd

RUN mkdir -p /opt/@tab
COPY tab_x64.so /opt/@tab/tab_x64.so

COPY --from=rust-builder /app/target/release/arma-bench-server /usr/local/bin/arma-bench-server

VOLUME /opt/servers/
EXPOSE 5672

CMD ["/usr/local/bin/arma-bench-server"]
