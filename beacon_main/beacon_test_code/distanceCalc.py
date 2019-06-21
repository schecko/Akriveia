import serial, os, sys, time, io

measure_power = -76.0
N = 2.0

a1 = serial.Serial('/dev/ttyUSB0', 115200)
a2 = serial.Serial('/dev/ttyUSB1', 115200)
#a3 = serial.Serial('/dev/ttyUSB3', 115200)

while True:
    msg1 = (a1.readline())
    msg2 = (a2.readline())
    #msg3 = (a3.readline())

    n1 = float(str(msg1).split('|')[-1].replace('\\r\\n\'', ''))
    n2 = float(str(msg2).split('|')[-1].replace('\\r\\n\'', ''))
    #n3 = float(str(msg3).split('|')[-1].replace('\\r\\n\'', ''))

    d1 = 10.0**((measure_power-(n1))/(10*N))
    d2 = 10.0**((measure_power-(n2))/(10*N))
    #d3 = 10.0**((measure_power-(n3))/(10*N))
    print("Distance1:", d1)
    print("Distance2:", d2)
    #print("Distance3:", d3)
    # Trilateration solver
    print("----------Trial----------")
    x = 0
    y = 0

    time.sleep(1)
