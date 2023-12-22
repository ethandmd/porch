import time
import sys
import cv2 as cv
from flask import Flask, Response
ipcam = sys.argv[1] if len(sys.argv) > 1 else exit(1)
app = Flask(__name__)
cap = cv.VideoCapture(ipcam)
if not cap.isOpened():
    print("Cannot open camera src.")
    exit(1)
cap.set(cv.CAP_PROP_BUFFERSIZE, 1)

def gen_frames(cap):
    
    while True:
        try:
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
            #time.sleep(0.01)
        except:
            cap.release()

@app.route('/cam30')
def cam30():
    return Response(gen_frames(cap), mimetype='multipart/x-mixed-replace; boundary=frame')

@app.route('/')
def index():
    return 'cam30:<br><img src="/cam30" /><br>'

app.run(host="0.0.0.0", port=3000, debug=False)
