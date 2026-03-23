# Sensor Health Analyzer

Collects data from sensors, makes them available for visualization and detecting anomalies, enabling predictive maintenance.

## System Architecture

![](docs/sys-arch.drawio.svg)

### Sensor Layer

- In a real-world scenario we have multiple sensors, each located in a different area, inside a machine/device
- Challenge: Each sensor could have a different network interface (e.g. WiFi, Ethernet, LoRaWAN), speak a different communication protocol, or other compatibility issues
- The goal is to support the most common networking protocols and data structures, so we can integrate as much sensors as possible
- As a first step, only MQTT is supported. The system can be extended in the future to support more protocols
- The [Zigbee2MQTT](https://www.zigbee2mqtt.io/) bridge is also supported, so you can collect sensor data from many of your Smart Home devices

### SensorVault

- The core of this project: Data from sensors should be made available for visualization, analyzing, detecting patterns and potential failures early (Keyword: "Predictive Maintenance")
- The **collector** service is responsible for receiving measurements from sensors and persisting them in a database
- All data received by sensors is saved in a [TimescaleDB](https://github.com/timescale/timescaledb)
- Grafana can be used to aggregate and visualize the sensor data on a Dashboard

## Project Structure

```shell
.
├── docs                # Documentation
├── core                # Core data structures
├── infra               # Traits + implementation of all interaction with external services, such as databases
├── collector           # Receives measurements from MQTT broker and saves them to database
├── mqtt_sim            # CLI application for testing - e.g. publish sensor measurements to the collector
├── mosquitto           # Config for the Mosquitto MQTT broker running in Docker
├── Cargo.toml          # Cargo manifest for this workspace
└── docker-compose.yml  # All services to run the whole system locally in Docker containers
```

## Get started

### Run services

Run all services, needed for the binaries:

```shell
docker-compose up
```

This will run:

- `mqtt_broker`: Mosquitto MQTT Broker, running at port 1883
  - `collector` subscribes to messages at topic `sensors/+/+` (wildcards mean "sensor_type/id")
  - `mqtt_sim` simulates an MQTT sensor and publishes messages to the broker
- `db`: PostgreSQL database, running at port 5432
  - Required for collector to save the sensor measurements
- `grafana`: For data visualization and alerting (and potentially much more in the future), UI running at [localhost:3000](http://localhost:3000/)

### Binaries

Run `collector` to receive sensor measurements via MQTT and save them in the database:

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
cargo run -p mqtt_sim send sensors/temperature/1 23.5
```

Of course you can also use your favorite MQTT client (like Eclipse Mosquitto):

```shell
mosquitto_pub -h localhost -p 1883 -t /sensors/temperature/1 -m "23.5"
```

...or as JSON in SenML format:

```json
{
  "n": "my-sensor-1",
  "v": 23.5,
  "u": "C"
}
```

```shell
mosquitto_pub -h $MQTT_HOST -p $MQTT_PORT -t /sensors/temperature/1 -m "{ 'n': 'my-sensor-1', 'v': 23.5, 'u': 'C' }"
```
