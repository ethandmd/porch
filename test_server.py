import cv2
import socket
import numpy as np

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', 8888))

count = 0
frames = []

while count < 9:
    count += 1
    data, addr = sock.recvfrom(2**18)
    frames.append(data)
    #if b"NEWFRAME" in data:
    #    count += 1
    #    frames.append(b'')
    #else:
        #if count >= 0:
        #    frames[count] += data

sock.close()

for i,j in enumerate(frames):
    print(i, len(j))
    if len(j) != 0 :
        buf = np.asarray(bytearray(j), dtype=np.uint8)
        jpg = cv2.imdecode(buf, cv2.IMREAD_COLOR)
        cv2.imwrite('images/image_' + str(i) + '.jpg', jpg)
