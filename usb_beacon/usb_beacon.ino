
void setup()
{
	Serial.begin(9600);
}

char i = 0;
void loop()
{
	char receiveByte = 0;
	if(Serial.available() > 0) {
		receiveByte = Serial.read();

    String msg = String("\nreceived ") + String(receiveByte, DEC);
		Serial.print(msg);
    i = 0;
	} else {
    String msg = String("\nwaiting ") + String(i++, DEC);
	  Serial.print(msg);
	}
	delay(1000);
}
