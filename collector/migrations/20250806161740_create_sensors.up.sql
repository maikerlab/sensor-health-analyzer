CREATE TABLE devices
(
    id          UUID                 DEFAULT gen_random_uuid() PRIMARY KEY,

    -- ID if device has a material and serial no.
    material_no VARCHAR(64),
    serial_no   VARCHAR(64),

    -- ID if device cannot be identified by material and serial no. (e.g. only by mqtt topic)
    custom_id   VARCHAR(255),

    name        TEXT        NOT NULL,
    device_type VARCHAR(64),
    location    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT device_has_identity CHECK (
        (material_no IS NOT NULL AND serial_no IS NOT NULL)
            OR custom_id IS NOT NULL
        ),
    -- Enforce uniqueness per identity type
    CONSTRAINT uq_device_serial
        UNIQUE (material_no, serial_no),
    CONSTRAINT uq_device_custom
        UNIQUE (custom_id)
);

CREATE TABLE sensors
(
    id          UUID                 DEFAULT gen_random_uuid() PRIMARY KEY,
    custom_id   VARCHAR(64),          -- e.g. sensors/temp/123 (MQTT)
    device_id   UUID,
    channel     VARCHAR(64) NOT NULL, -- e.g. temperature, signal_strength
    unit        VARCHAR(20),          -- "°C" | "%" | "dB"
    description VARCHAR(255),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_sensors_devices FOREIGN KEY (device_id) REFERENCES devices (id) ON DELETE CASCADE
);

CREATE TABLE sensor_data
(
    time      TIMESTAMPTZ      NOT NULL,
    sensor_id UUID             NOT NULL,
    value     DOUBLE PRECISION NOT NULL,
    CONSTRAINT fk_sensor_data_sensors FOREIGN KEY (sensor_id) REFERENCES sensors (id)
) WITH (
      tsdb.hypertable,
      tsdb.partition_column = 'time'
      );
