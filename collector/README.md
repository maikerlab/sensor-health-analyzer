# Collector

## Supported ingestion handlers

### MQTT

Identification of sensors is done by the MQTT topic.
At the moment, only "raw decoding" is performed, which means the collector expects just plain float values in the payload.

## Migrations

Show info about migrations:

```shell
sqlx migrate info
```

Add a new migration script (`-r` flag for both up- and down-script):

```shell
sqlx migrate add -r <name>
```

Run pending migrations:

```shell
sqlx migrate run
```

Revert latest migration

```shell
sqlx migrate revert
```
