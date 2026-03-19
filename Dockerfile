# SAGE — Decentralized AI Node
# docker run -v ~/.sage:/root/.sage ghcr.io/caryyon/sage

FROM rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev libasound2-dev libv4l-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cargo build --release --bin sage

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sage /usr/local/bin/sage

ENV SAGE_HOME=/root/.sage
RUN mkdir -p /root/.sage/bin /root/.sage/data /root/.sage/peers

EXPOSE 7433

ENTRYPOINT ["sage"]
CMD ["node", "start", "--chat-port", "7433"]
