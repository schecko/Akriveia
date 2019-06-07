bool system_on = false;
String input = "";

void setup() {
  Serial.begin(9600);
  Serial.println("Enter 'start' to start sending data");
  Serial.println("Enter 'stop' to stop sending data");
}

void loop() {
  
  if (Serial.available()>0) {
    
    input = Serial.readString();
    
    if (input.indexOf("start")>=0){
      Serial.println("Start Recieved, Start sending data:");
      system_on = true;
    }
    else if (input.indexOf("end")>=0){
      Serial.println("End Recieved, stop sending data:");
      system_on = false;
    }
  }

  if (system_on==true){
    Serial.println("DATA");
  }

  delay(500);
}
