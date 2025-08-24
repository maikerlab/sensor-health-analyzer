FROM rust:1.88-bookworm AS builder

WORKDIR /workspace

COPY .. .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin dispatcher
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates gcc libc6-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*
ENV IOT_DATABASE_URL="postgres://iot:sensor@db/sensor_db"
ENV IOT_NATS_URL="nats://nats:4222"
ENV IOT_MQTT_HOST="mqtt_broker"
ENV IOT_MQTT_PORT="1883"
COPY --from=builder /workspace/target/release/dispatcher /app/dispatcher
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY ../db/migrations /app/migrations
COPY ../dispatcher/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
