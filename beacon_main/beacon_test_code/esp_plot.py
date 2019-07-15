import time, sys, os, threading
import serial, io
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib import style

measure_power = -76.0
N = 2.0

x_1 = 0.0
y_1 = 0.0

x_2 = 3.0
y_2 = 0.0

x_3 = 0.0
y_3 = 3.0

data = 0.0
index = 0.0

def data_maker(a1,a2,a3):
    msg1 = (a1.readline())
    msg2 = (a2.readline())
    msg3 = (a3.readline())
    n1 = float(str(msg1).split('|')[-1].replace('\\r\\n\'', ''))
    n2 = float(str(msg2).split('|')[-1].replace('\\r\\n\'', ''))
    n3 = float(str(msg3).split('|')[-1].replace('\\r\\n\'', ''))
    d1 = 10.0 ** ((measure_power - (n1)) / (10 * N))
    d2 = 10.0 ** ((measure_power - (n2)) / (10 * N))
    d3 = 10.0 ** ((measure_power - (n3)) / (10 * N))
	
    A = -2*x_1 + 2*x_2

    B = -2*y_1 + 2*y_2
    
    C = d1*d1 - d2*d2 - x_1*x_1 + x_2*x_2 - y_1*y_1 + y_2*y_2
    
    D = -2*x_2 + 2*x_3
    
    E = -2*y_2 + 2*y_3
 
    F = d2*d2 - d3*d3 - x_2*x_2 + x_3*x_3 - y_2*y_2 + y_3*y_3


    x = (C*E - F*B) / (E*A - B*D)

    y = (C*D - A*F) / (B*D - A*E)

	index = x
    data = y

    time.sleep(1)


def plotter():
    def animate(i):
        x = index
        y = data
        ax1.clear()
        ax1.set_ylim(bottom=0, top=10)
        ax1.set_xlim(left=0, right=10)
        ax1.plot(x, y, linewidth=1, c="#32CD32", linestyle='-')

    style.use('seaborn-whitegrid')
    fig = plt.figure(num=1, figsize=(10,4), dpi=100)
    ax1 = fig.add_subplot(1, 1, 1)
    ani = animation.FuncAnimation(fig, animate, interval=1000)
    plt.grid(True)
    plt.show()

if __name__ == "__main__":


    a1 = serial.Serial('COM4', 115200)
    a2 = serial.Serial('COM9', 115200)
    a3 = serial.Serial('COM10', 115200)

    print("\n-Start-\n")
    try:
        t1 = threading.Thread(target=data_maker(a1,a2,a3))
        t2 = threading.Thread(target=plotter)
        t1.start()
        time.sleep(5)
        t2.start()
    except KeyboardInterrupt:
        t2.join()


