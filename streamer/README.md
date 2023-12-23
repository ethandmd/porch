this works on pi:
```
"( libcamerasrc ! video/x-raw, width=640, height=480, framerate=30/1 ! videoconvert ! x264enc ! queue ! h264parse ! rtph264pay name=pay0 pt=96 config-interval=1 )"
```
this works on pc (w/o libcamerasrc):
```
"( v4l2src device=/dev/video0 ! video/x-raw, width=640, height=480, framerate=30/1 ! videoconvert ! queue ! x264enc tune=zerolatency ! rtph264pay name=pay0 pt=96 )"
```
