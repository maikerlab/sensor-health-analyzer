FROM rust:1.88-bookworm AS builder

WORKDIR /workspace

COPY .. .
RUN cargo build --release --bin collector

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates gcc libc6-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*
ENV IOT_NATS_URL="nats://nats:4222"
ENV IOT_MQTT_HOST="mqtt_broker"
ENV IOT_MQTT_PORT="1883"
COPY --from=builder /workspace/target/release/collector /app/collector

ENTRYPOINT ["/app/collector"]
