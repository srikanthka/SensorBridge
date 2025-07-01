SensorBridge is a Rust-based embedded firmware designed for the ESP32 microcontroller.
It provides a bridge between UART-connected environmental sensors (e.g., temperature, humidity, CO₂, NO₂) and Bluetooth Low Energy (BLE) clients.
The firmware supports real-time sensor data acquisition, BLE notifications, and status indication through LED blinking.

Main Features
1. UART Data Acquisition
Reads data from one or more sensors connected over UART (e.g., temp, humidity, CO₂, NO₂ sensors).
Parses and prepares the data for BLE transmission.

2. BLE GATT Server
Acts as a BLE peripheral.
Exposes sensor data via custom BLE characteristics.
Sends notifications to subscribed BLE clients when new UART data arrives.

3. LED Blinking
Blinks an on-board LED periodically to indicate system status (e.g., heartbeat, error state).

4. UART-to-BLE Notification Bridge
Whenever new data is read on UART, it is immediately sent to BLE clients that have subscribed.

5. Efficient, safe Rust code
Leverages esp-idf-hal and esp-idf-svc.
Memory safe, no heap fragmentation.

SensorBridge: ESP32 Rust firmware to collect environmental sensor data via UART
and broadcast it over BLE using notifications.
