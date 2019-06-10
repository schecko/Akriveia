
#include <WiFi.h>
#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEScan.h>
#include <BLEAdvertisedDevice.h>

int beaconScanTime = 4;
uint8_t bufferIndex = 0;
BeaconData buffer[50];

typedef struct {
  char address[17]; 
  int rssi;
} BeaconData;


class MyAdvertisedDeviceCallbacks : public BLEAdvertisedDeviceCallbacks {
public:
	void onResult(BLEAdvertisedDevice advertisedDevice) {
	    extern uint8_t bufferIndex;
	    extern BeaconData buffer[];
	    if(bufferIndex >= 50) { return; }
	    
	    // RSSI
	    if(advertisedDevice.haveRSSI()) {
	      buffer[bufferIndex].rssi = advertisedDevice.getRSSI();
	    } 
	    else { 
	    	buffer[bufferIndex].rssi =  0; 
	    }
	    
	    // MAC is mandatory for BT to work
	    strcpy (buffer[bufferIndex].address, advertisedDevice.getAddress().toString().c_str());
	    bufferIndex++;

	    // Print
	    Serial.printf("MAC: %s \n", advertisedDevice.getAddress().toString().c_str());
	    Serial.printf("name: %s \n", advertisedDevice.getName().c_str());
	    Serial.printf("RSSI: %d \n", advertisedDevice.getRSSI());
	}
};


void setup(){
  Serial.begin(115200);
  BLEDevice::init("");
}


void ScanBeacons() {
  delay(500);
  BLEScan* pBLEScan = BLEDevice::getScan();
  MyAdvertisedDeviceCallbacks cb;
  pBLEScan->setAdvertisedDeviceCallbacks(&cb);
  pBLEScan->setActiveScan(true);
  BLEScanResults foundDevices = pBLEScan->start(beaconScanTime);

  Serial.print("Devices found: ");
  for (uint8_t i = 0; i < bufferIndex; i++) {
    Serial.print(buffer[i].address);
    Serial.print(" : ");
    Serial.println(buffer[i].rssi);
  }
  
  pBLEScan->stop();
  delay(500);
  Serial.println("Scan done!");
}


void loop() {

  ScanBeacons();


}
