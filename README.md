# Porch

## Usage
Use `rtsp-server` on the (raspberry pi) IP cam with gstreamer installed.
Run `stream.py` on the rtsp client pointed at the IP cam. Navigate to localhost:5000:
```
python3 stream.py "rtsp://192.168.1.9:8554/cam"
```
