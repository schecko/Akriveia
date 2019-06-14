/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon Sketch
 */

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEAdvertisedDevice.h>
#include <BLEScan.h>

#define MAX_CLIENT 1

bool system_on = false;
String input = "";

typedef struct {
  char macAddr[17];
  int rssi;
} clientInfo;

int clientIndex = 0;
int scanTime = 1;
clientInfo buffer[MAX_CLIENT];

class MyAdvertisedDeviceCallbacks : public BLEAdvertisedDeviceCallbacks {
  public:

  // Invoked per unique device found
  void onResult(BLEAdvertisedDevice advertisedDevice) {
    extern int clientIndex;
    extern clientInfo buffer[];
    if(clientIndex >= MAX_CLIENT) {
      return;
    }

    if(advertisedDevice.haveRSSI()) {
      buffer[clientIndex].rssi = advertisedDevice.getRSSI();
    }
    else {
      buffer[clientIndex].rssi = 0;
    }

    strcpy(buffer[clientIndex].macAddr, advertisedDevice.getAddress().toString().c_str());
    clientIndex++;

    Serial.print(advertisedDevice.getAddress().toString().c_str());
    Serial.print("|");
    Serial.print(advertisedDevice.getName().c_str());
    Serial.print("|");
    Serial.print(advertisedDevice.getRSSI());
    Serial.println("");
  }
};


void setup() {
  Serial.begin(115200);
  Serial.println("Enter 'start' to start sending data");
  Serial.println("Enter 'stop' to stop sending data");
  BLEDevice::init("Beacon");

}

void clientScan() {
  BLEScan* beacon = BLEDevice::getScan();
  MyAdvertisedDeviceCallbacks cb;
  beacon->setAdvertisedDeviceCallbacks(&cb);
  beacon->setActiveScan(true);
  BLEScanResults foundClients = beacon->start(scanTime);
  
  /*Serial.println("Clients found: ");

  for(int i = 0; i < clientIndex; i++) {
    Serial.print(buffer[i].macAddr);
    Serial.print(" : ");
    Serial.println(buffer[i].rssi);
  }

  beacon->stop();
  Serial.println("Scan complete!");
  */
}



void loop() {
  if (Serial.available() > 0) {
    
    input = Serial.readString();

    if (input.indexOf("start") >= 0) {
      Serial.println("Starting...");
      system_on = true;
    }
    else if (input.indexOf("end") >= 0) {
      Serial.println("Ending...");
      system_on = false;
    }
  }

  if (system_on){
     clientScan();
     clientIndex = 0;
    }
  

}
