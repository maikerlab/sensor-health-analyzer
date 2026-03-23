# Requirements

Keywords:

- **SHOULD**: Must be fulfilled
- **CAN**: Optional requirement

## SHA-001 Collecting and persisting sensor data

The system SHOULD collect measurements from various sources and save them.
The datasource where sensor data is stored SHOULD be made accessible for 3rd party systems, so the data can be aggregated and evaluated.

Sensor data could be sent to the collector in various forms:

- Live sensor data sent over the network, e.g. over MQTT, industrial protocols like OPC-UA, etc.
- Sensor data read out via a serial interface of the device
- Manual measurements, sent via REST-API or entered with shell commands

The system SHOULD be easily extensible with adapters for all kinds of sensor data sources.

## SHA-002 Support collecting data from MQTT sensors

The system SHOULD be able to collect live sensor data sent via MQTT, with various data structures.
One is that the measurement values are sent as a string in the payload, but also payloads sent by the Zigbee2MQTT bridge must be supported.

## SHA-003 Evaluate sensor data

The system SHOULD be able to evaluate new sensor data and tell the user how to interpret the data.

### Example: Laser degradation of a LiDAR sensor

1. Signal strength at calibration step during production: `100mV` → saved by **collector**
2. Customer returned it to the manufacturer after 1 week of usage
3. Sensor is without errors, but needs to be recalibrated before sold again → Signal strength: `95mV` → saved by **collector**
4. "Predictive maintenance step" after calibration:
    - The SensorHealthAnalyzer takes the current signal strength (95mV), the current age and total operating hours of the device and evaluates it
    - For past data of sensors of this type, it was found that the drop of signal strength (5%) just during 1 week of usage is unusually high
    - This information is output to the user with the recommendation to replace the laser module and recalibrate the LiDAR sensor
