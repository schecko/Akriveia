/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon Sketch
 */

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEAdvertisedDevice.h>
#include <BLEScan.h>

bool system_on = false;
String input = "";
int scanTime = 1;

char *mac_list[] = {"cc:50:e3:9c:4c:c2",
                    "cc:50:e3:9c:48:86"};




class MyAdvertisedDeviceCallbacks : public BLEAdvertisedDeviceCallbacks {
  public:
  
  void onResult(BLEAdvertisedDevice advertisedDevice) {
    for (int i = 0; i < sizeof(mac_list)/sizeof(mac_list[0]); i++){
      if (advertisedDevice.getAddress().toString()==mac_list[i]){
        Serial.print(advertisedDevice.getName().c_str());
        Serial.print("|");
        Serial.print(advertisedDevice.getAddress().toString().c_str());
        Serial.print("|");
        Serial.print(advertisedDevice.getRSSI());
        Serial.println("");
      }
    }
  }
};

void setup() {
  Serial.begin(115200);
  Serial.println("Enter 'start' to start sending data");
  Serial.println("Enter 'stop' to stop sending data");
  BLEDevice::init("Beacon");
}


void loop() {
 
  if (Serial.available() > 0) {
    input = Serial.readString();
    if (input.indexOf("start") >= 0) { Serial.println("Starting..."); system_on = true; }
    else if (input.indexOf("end") >= 0) { Serial.println("Ending..."); system_on = false; }
  }

  if (system_on) {
    BLEScan* beacon = BLEDevice::getScan();
    MyAdvertisedDeviceCallbacks cb;
    beacon->setAdvertisedDeviceCallbacks(&cb);
    beacon->setActiveScan(true);
    BLEScanResults foundClients = beacon->start(scanTime);
  }
}
