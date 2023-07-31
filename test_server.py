import cv2
import socket
import numpy as np

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', 8888))

count = -1
frames = []

while count < 9:
    data, addr = sock.recvfrom(2**17)
    if 'NEWFRAME'.encode() in data:
        count += 1
        frames.append(b'')
    else:
        if count >= 0:
            frames[count] += data

sock.close()

i = 0
for image in frames:
    buf = np.asarray(bytearray(image), dtype=np.uint8)
    image = cv2.imdecode(buf, cv2.IMREAD_COLOR)
    cv2.imwrite('images/image' + str(i) + '.jpg', image)
