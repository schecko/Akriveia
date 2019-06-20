bool system_on = false;
String input = "";

// responses
const String ack = "ack";
const String nak = "nak";

// commands
const String command_start = "start";
const String command_end = "end";

int data = 0;

void setup() {
  Serial.begin(9600);
}

void loop() {

  if (Serial.available()>0) {

    input = Serial.readString();

    if (input.indexOf(command_start)>=0){
      Serial.println(ack);
      system_on = true;
    }
    else if (input.indexOf(command_end)>=0){
      Serial.println(ack);
      system_on = false;
    } else {
	  Serial.println(nak);
    }
  }

  if (system_on==true){
    Serial.println(data);
	data++;
  }

  delay(500);
}
