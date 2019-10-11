
int n_tag = 5;
int counter = 0;
int index_ = 0;
int count_per_index = 10;
int max_count = count_per_index  * n_tag - 1;

byte A[] = {B00000001, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};
byte B[] = {B00000010, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};


void setup() {
  Serial.begin(9600);
}

void loop() {
  if (counter == 0) index_ = 0;
  else index_ = floor(counter / count_per_index % counter) ;



  
  Serial.println(String(counter) + "|" + String(index_));





  if (counter >= max_count) counter = 0;
  else counter++;

  delay(100);
}
