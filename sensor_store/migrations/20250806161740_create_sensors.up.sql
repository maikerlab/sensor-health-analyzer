CREATE TABLE sensors
(
    id       SERIAL PRIMARY KEY,
    type     VARCHAR(50),
    location VARCHAR(50)
);

CREATE TABLE sensor_data
(
    time      TIMESTAMPTZ NOT NULL,
    sensor_id INTEGER,
    value     DOUBLE PRECISION,
    unit      VARCHAR(50),
    FOREIGN KEY (sensor_id) REFERENCES sensors (id)
) WITH (
      tsdb.hypertable,
      tsdb.partition_column = 'time'
      );
