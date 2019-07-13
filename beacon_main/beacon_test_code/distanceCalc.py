import serial, os, sys, time, io

measure_power = -76.0
N = 2.0

x_1 = 0.0
y_1 = 0.0

x_2 = 3.0
y_2 = 0.0

x_3 = 0.0
y_3 = 3.0


a1 = serial.Serial('COM4', 115200)
a2 = serial.Serial('COM9', 115200)
a3 = serial.Serial('COM10', 115200)

while True:
    msg1 = (a1.readline())
    msg2 = (a2.readline())
    msg3 = (a3.readline())

    n1 = float(str(msg1).split('|')[-1].replace('\\r\\n\'', ''))
    n2 = float(str(msg2).split('|')[-1].replace('\\r\\n\'', ''))
    n3 = float(str(msg3).split('|')[-1].replace('\\r\\n\'', ''))

    d1 = 10.0**((measure_power-(n1))/(10*N))
    d2 = 10.0**((measure_power-(n2))/(10*N))
    d3 = 10.0**((measure_power-(n3))/(10*N))

    # Trilateration solver

    A = -2*x_1 + 2*x_2

    B = -2*y_1 + 2*y_2
    
    C = d1*d1 - d2*d2 - x_1*x_1 + x_2*x_2 - y_1*y_1 + y_2*y_2
    
    D = -2*x_2 + 2*x_3
    
    E = -2*y_2 + 2*y_3
 
    F = d2*d2 - d3*d3 - x_2*x_2 + x_3*x_3 - y_2*y_2 + y_3*y_3


    x = (C*E - F*B) / (E*A - B*D)

    y = (C*D - A*F) / (B*D - A*E)

    print(str(x) + '|' str(y))

    time.sleep(1)
