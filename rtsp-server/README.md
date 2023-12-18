## References

https://github.com/GStreamer/gst-rtsp-server/blob/master/examples/test-readme.c
https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/main/examples/src/bin/rtsp-server.rs

## Usage on rpi 3 B:
```
libcamera-vid -n -t 0 -o - | ./rtsp-server "( fdsrc fd=0 ! h264parse ! rtph264pay name=pay0 pt=96 )"
```
