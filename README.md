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
├── mqtt_worker         # Receives measurements from MQTT broker and forwards it to NATS
├── dispatcher          # Receives measurements from NATS and saves them in the database
├── sensor_store        # Provides methods for data access
├── Cargo.toml          # Workspace members, dependencies, ...
└── docker-compose.yml  # Services to run the whole system locally
```

## Get started

Run depending services - Database and NATS server:

```shell
docker-compose up
```

Run `dispatcher` to receive sensor measurements:

```shell
cargo run -p dispatcher
```

Run `mqtt_worker` to subscribe to sensor measurements from MQTT and forward it to NATS and Dispatcher:

```shell
cargo run -p mqtt_worker
```

Publish sample message as sensor simulation:

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
