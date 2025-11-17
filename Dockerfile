FROM rust:1.90 AS builder

WORKDIR /usr/src/app
COPY src/ src/
COPY Cargo.toml Cargo.toml

RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/byond-ping-websocket-server /usr/local/bin/byond-ping-websocket-server

CMD ["byond-ping-websocket-server"]
