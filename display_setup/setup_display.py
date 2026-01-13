import serial

port = serial.Serial("/dev/ttyUSB0", 19200, timeout=1)
port.write([254, 195, 2, 1])
port.write([254, 195, 1, 0])
port.write([254, 195, 3, 0])
port.write([254, 195, 4, 0])
port.write([254, 195, 5, 0])
port.write([254, 195, 6, 0])
