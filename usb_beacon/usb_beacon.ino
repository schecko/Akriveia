
void setup()
{
	Serial.begin(9600);
}

char i = 0;
void loop()
{
	/*char receiveByte = 0;
	if(Serial.available() > 0) {
		receiveByte = Serial.read();

		// say what you got:
		Serial.print("I received: ");
		Serial.println(receiveByte, DEC);
	}
	*/


	Serial.print((int)i++);
	delay(1000);
}
