# IoT Gateway

A demo project as a showcase about how to implement an IoT gateway in Rust and general architectural considerations in the sector of IoT.

## System Architecture

![](docs/sys-arch.drawio.svg)

### Sensors

- The goal is to be as flexible as possible and support various sensor types and the most common networking protocols and data structures
- Sensors must be able to store cryptographic keys. The keys can either be generated on the device or provisioned by an external system.
- Sensors sign measurements with their private key and send it to a dedicated worker

### Workers

- Are responsible for receiving measurements from sensors
- Can support receiving measurements by any communication method, like MQTT, HTTP/REST, LoRaWAN etc...
- As a first start, the `mqtt_worker` is implemented, where sensors can publish measurements to
- Any worker must convert the sensor values (which could be of different formats) to a `SenMLRecord`, encode it in the CBOR format and forward the package to the NATS server
- Benefit: Ideally sensors already send their measurements in the SenML format, so we can just forward it

### Dispatcher

The dispatcher subscribes to messages sent to NATS and saves them in the TimeriesDB database.

### Registry

As a further step, an application (WebApp and/or CLI) could be developed for the following use cases:

- Register new sensors
- Manage registered sensors (e.g. deactivate, set location)
- ...

### Database

TimeseriesDB is used as the database for persisting sensor values and device registry.

### NATS Server

A NATS server must be running for receiving sensor measurements from the workers. To ensure a good QoS (Quality of Service), NATS is run with the JetStream option, which enables buffering of received messages.
Messages which are not received by any subscribers are held back and as soon as a subscriber goes online, it receives the buffered messages.
This ensures that no sensor measurements are lost and sensors are not dependent on the availability of the dispatcher.


## Project Structure

```shell
.
├── docs                # Documentation
├── common              # Common data structures shared across the workspace
├── db                  # Connection and queries to the PostgreSQL database
├── messaging           # Connection and methods to push/pull messages to/from NATS
├── dispatcher          # Receives measurements from NATS and saves them in the database
├── mqtt_worker         # Receives measurements from MQTT broker and forwards it to NATS
├── sensor_simulator    # CLI application for testing - e.g. publish sensor measurements to the mqtt_worker
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
  - `sensor_simulator` publishes its messages to the broker
  - `mqtt_worker` subscribes to messages at topic `sensors/+/+` (wildcards mean "sensor_type/payload")
- `db`: PostgreSQL database, running at port 5432
  - Required for dispatcher to save the sensor measurements
- `nats`: NATS Messaging Broker, running at port 4222
  - Required for workers to publish SenML records to
  - Required for dispatcher to receive SenML records
- `grafana`: For data analysis and alerting (and potentially much more in the future), UI running at [localhost:3000](http://localhost:3000/)

### Binaries

Run `dispatcher` to receive sensor measurements:

```shell
cargo run -p dispatcher
```

Run `mqtt_worker` to subscribe to sensor measurements from MQTT and forward it to NATS and Dispatcher:

```shell
cargo run -p mqtt_worker
```

### Testing

For simplicity, you can just run the `sensor_simulator` binary:

```shell
cargo run -p sensor_simulator loop temp
```

...or send a custom message to a topic:

```shell
cargo run -p sensor_simulator send sensors/temp/1 23.5
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
