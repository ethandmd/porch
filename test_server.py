import cv2
import socket

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', 8888))

count = -1
frames = []

while count < 10:
    data, addr = sock.recvfrom(1024)
    if 'NEWFRAME'.encode() in data:
        count += 1
        frames.append(b'')
    else:
        if count >= 0:
            frames[count] += data

sock.close()
