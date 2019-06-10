# Anchor Beacon

## Core:
https://github.com/espressif/arduino-esp32

## Examples:
- https://github.com/nkolban/ESP32_BLE_Arduino
- https://github.com/jarkko-hautakorpi/iBeacon-indoor-positioning-demo
- https://randomnerdtutorials.com/esp32-bluetooth-low-energy-ble-arduino-ide/


07/06/2019

- Used BLE_client.ino and BLE_server.ino for a beacon and an ID tag respectively.
- Included BLEClient.h in the beacon code to use the getRssi() function. ie. int rssiValue = pClient->getRssi(); Serial.println(rssiValue);
- Integer RSSI values in dBm were shown at serial output on the ID tag side
- Performed (ad-hoc) simple range testing
- Deduced 1 meter RSSI (Measured Power) as ~-77dBm (requires more research)

Note: The boot button on the ESP32 module must be pressed during the code uploading process from Arduino IDE.