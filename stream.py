import time
import sys
import cv2 as cv
from flask import Flask, Response
ipcam = sys.argv[1] if len(sys.argv) > 1 else exit(1)
app = Flask(__name__)
cap = cv.VideoCapture(ipcam)

def gen_frames():
    while True:
        ret, frame = cap.read()
        if not ret:
            break
        ret, frame = cv.imencode('.jpg', frame)
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

app.run()
