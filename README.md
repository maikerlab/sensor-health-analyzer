# Sensor Health Analyzer

Collects data from sensors, makes them available for visualization and detecting anomalies, enabling predictive maintenance.

## System Architecture

![](docs/sys-arch.drawio.svg)

### Sensor Layer

- The goal is to be as flexible as possible and support various sensor types and the most common networking protocols and data structures
- As a first step, only MQTT is supported
- There is a nice integration in HomeAssistant (Zigbee2MQTT), where we can collect sensor data from our Smart Home devices

### Workers

- Are responsible for receiving measurements from sensors
- Can support receiving measurements by any communication method, like MQTT, HTTP/REST, LoRaWAN etc...
- As a first start, the `collector` is implemented, where sensors can publish measurements to
- Any worker must convert the sensor values (which could be of different formats) to a `SenMLRecord`, encode it in the CBOR format and forward the package to the NATS server
- Benefit: Ideally sensors already send their measurements in the SenML format, so we can just forward it

### Database

TimeseriesDB is used as the database for persisting sensor values.

## Project Structure

```shell
.
├── docs                # Documentation
├── common              # Common data structures shared across the workspace
├── dispatcher          # Receives measurements from NATS and saves them in the database
├── collector         # Receives measurements from MQTT broker and forwards it to NATS
├── mqtt_sim    # CLI application for testing - e.g. publish sensor measurements to the collector
├── mosquitto           # Config for the Mosquitto MQTT broker running in Docker
├── Cargo.toml          # Workspace members, dependencies, ...
└── docker-compose.yml  # Services to run the whole system locally in Docker
```

## Get started

### Run services

Run all services, needed for the binaries:

```shell
docker-compose up
```

This will run:

- `mqtt_broker`: Mosquitto MQTT Broker, running at port 1883
  - `collector` subscribes to messages at topic `sensors/+/+` (wildcards mean "sensor_type/payload")
  - `mqtt_sim` simulates an MQTT sensor and publishes messages to the broker
- `db`: PostgreSQL database, running at port 5432
  - Required for collector to save the sensor measurements
- `grafana`: For data analysis and alerting (and potentially much more in the future), UI running at [localhost:3000](http://localhost:3000/)

### Binaries

Run `dispatcher` to receive sensor measurements:

```shell
cargo run -p dispatcher
```

Run `collector` to subscribe to sensor measurements from MQTT and forward it to NATS and Dispatcher:

```shell
cargo run -p collector
```

### Testing

For simplicity, you can just run the `mqtt_sim` binary:

```shell
cargo run -p mqtt_sim loop temp
```

...or send a custom message to a topic:

```shell
cargo run -p mqtt_sim send sensors/temp/1 23.5
```

Of course you can also use your favorite MQTT client (like Eclipse Mosquitto):

```shell
mosquitto_pub -h localhost -p 1883 -t /sensors/temp/1 -m "23.5"
```

...or as JSON:

```json
{
  "n": "my-sensor-1",
  "v": 23.5,
  "u": "C"
}
```

```shell
mosquitto_pub -h $MQTT_HOST -p $MQTT_PORT -t /sensors/temp/1 -m "{ 'n': 'my-sensor-1', 'v': 23.5, 'u': 'C' }"
```
