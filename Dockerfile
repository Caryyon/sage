FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin sage-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sage-cli /usr/local/bin/sage
COPY --from=builder /app/data/training/qa-corpus.txt /app/data/training/qa-corpus.txt

RUN mkdir -p /root/.sage

EXPOSE 4001 19175

ENTRYPOINT ["sage"]
CMD ["chat"]
