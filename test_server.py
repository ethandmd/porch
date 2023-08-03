import cv2 as cv
import socket
import time
import numpy as np
from flask import Flask, Response

app = Flask(__name__)
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind(('127.0.0.1', 8888))

#fourcc = cv.VideoWriter_fourcc(*'HVID')
#out = cv.VideoWriter('output.avi', fourcc, 15.0, (1280, 720))

#count = 0
#frames = []

def gen_frames():
    count = 0
    frames = []
    while True:
        data, addr = sock.recvfrom(2**18)
        #buf = np.asarray(bytearray(data), dtype=np.uint8)
        #buf = np.frombuffer(data, dtype=np.uint8)
        #jpg = cv.imdecode(buf, cv.IMREAD_COLOR)
        #ret, jpg = cv.imencode('.jpg', buf)
        #frame = jpg.tobytes()
        #cv.imwrite('images/image_' + str(count) + '.jpg', jpg)
        #out.write(buf)
        #frames.append(buf)
        #cv.imshow('frame', buf)
        count += 1
        yield(
                b'--frame\r\n'
                b'Content-Type: image/jpeg\r\n'
                b'Content-Length: ' + f"{len(data)}".encode() + b'\r\n'
                b'\r\n' + data + b'\r\n'
        )
        time.sleep(0.01)

@app.route('/feed')
def feed():
    return Response(gen_frames(), mimetype='multipart/x-mixed-replace; boundary=frame')

@app.route('/')
def index():
    return 'image:<br><img src="/feed" />'

app.run(debug=True)

sock.close()
#out.release()
