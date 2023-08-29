import cv2 as cv
import time
import numpy as np
from flask import Flask, Response

app = Flask(__name__)
cap = cv.VideoCapture('udp://192.168.1.21:8888')

def gen_frames():

    while True:
        ret, buff = cap.read()
        if not ret:
            break
        ret, frame = cv.imencode('.jpg', buff)
        data = frame.tobytes()
        yield(
                b'--frame\r\n'
                b'Content-Type: image/jpeg\r\n'
                b'Content-Length: ' + f"{len(data)}".encode() + b'\r\n'
                b'\r\n' + data + b'\r\n')
        time.sleep(0.01)

@app.route('/feed')
def feed():
    return Response(gen_frames(), mimetype='multipart/x-mixed-replace; boundary=frame')

@app.route('/')
def index():
    return 'image:<br><img src="/feed" />'

app.run(debug=True)
