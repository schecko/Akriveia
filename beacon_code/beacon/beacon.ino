/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon Sketch
 */

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEAdvertisedDevice.h>
#include <BLEScan.h>

#define MAX_CLIENT 1

RTC_DATA_ATTR int bootCount = 0;

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
  BLEDevice::init("Beacon");

/*
  ++bootCount;
  Serial.println("Boot number: " + String(bootCount));
  print_wakeup_reason();
  esp_sleep_enable_ext0_wakeup(GPIO_NUM_33, 1);

  Serial.println("Going to sleep now");
  delay(1000);
  esp_deep_sleep_start();
  Serial.println("This will never be printed");
*/
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
  clientScan();

  clientIndex = 0;

}