# Camserv

## Setup:
```bash
sudo apt install {g++, clang} # up to you
sudo apt install meson ninja-build pkg-config
sudo apt install openssl
sudo apt install libevent-dev # so we can use the "cam" app for debugging
sudo apt install libyaml-dev python3-yaml python3-ply python3-jinja2

git clone https://git.libcamera.org/libcamera/libcamera.git
cd libcamera
meson setup build
ninja -C build install

# Now, in current shell for running camserv:
export GST_PLUGIN_PATH=$(pwd)/build/src/gstreamer # And now you can use libcamerasrc :)
```

## Debug:
Set `GST_DEBUG=LEVEL` where LEVEL is 1,2,3,...7? Also, can do `_=*:2`.

Get pipeline viz when running debug:
```
sudo apt install graphviz
dot -Tsvg pipeline.dot > pipeline.svg
```

### Goals:
This crate detects motion, takes stills, and streams live video on demand.

It listens on a persistent HTTPS connection with the C2 server and takes actions 
based on received commands.

It runs simple and continuous image analysis on captured frames and on select events transmits event data to C2 server.

### Trappings
- Websocket / HTTP client with TLS and JWT auth (`tokio-tungstenite`, `tokio_rustls`, `hyper`, `reqwest`, `jsonwebtoken`).
- Live video stream (`webrtc-rs/rtp`, `webrtc-rs/sdp`, `gstreamer-rs`, `std::process::Command`, ...?)
- Camera capture (`opencv`, `gstreamer-rs`, `v4l/v4l2-sys-mit/rscam`, ...?)
- Image analysis (`image`, `opencv`, ...?)

NOTE: Must be mindful of TLS flavor. Currently going for rustls-webpki-roots...

### Functional Components
- Long running websocket connection which recieves commands that trigger actions (live stream, take image, etc...).
- Long running camera capture and analysis which can trigger action.

### Design
'Builder' creational pattern for device context.
'Command' behavioural pattern for receiving C2 commands.
Stream image frames with userptr if supported to avoid lots of copying data.

Main task runs camera capture and analysis, with a websocket running the background and spawning commands.

### Device Lifecycle 
1) User receives device, connects power supply and mounts device, proceed to `2`.

2) Device Setup:
    a) Self initialization, build `DeviceContext`; What cameras am I connected to? What is my system? etc...
        - Queries system camera connections
        - Confirms system camera works with simple test capture
        - Possibly conducts calibration
        - Confirms image analysis logic is functional; if no, user decides proceed yes/no or debug.
    b) Registers itself with C2 node as %USER's camera%ID.
        - Failure ? 4 : _ 
        - Submits initial test capture for review
        - Updates `DeviceContext` and saves to disk in porch config location. 
    c) Sets up command socket with C2 node.
    d) Begin normal operation.

3) Device power cycle:
    a) Query disk for last saved `DeviceContext`:
        - Exists ? Load context and run `2` with loaded context : Run `2` from blank context.

4) No C2 connectivity:
    a) Internet connectivity ? c,d : b.
    b) LAN connectivity ? d : `5`.
    c) Ping last known C2 address at progressively longer intervals ? `2` : c
    d) Local service at camera%ID.local:80 hostname.

5) No network connectivity:
    0) Device setup guide with self hosted wifi AP or ethernet lan and web UI at `porch_cam.local:80`
    a) Ping last known LAN gateway and remote address (e.g. 8.8.8.8) at progressively longer intervals ? `2` : a
    b) Detect nearby porch camera self hosting wifi AP ? c : d.
    c) Attempt to connect to nearby porch camera wifi AP up to 3x; success ? `fallback_register_procedure` : d.
    d) Start self hosted wifi AP and serve on demand live feed, captured stills from motion detection at camera%ID.local:80.

USEFUL:
- [`tokio_tungstenite`](https://docs.rs/tokio-tungstenite/0.21.0/tokio_tungstenite/)
- [`tokio_rustls`](https://docs.rs/tokio-rustls/latest/tokio_rustls/index.html)
- [`hyper`](https://docs.rs/hyper/latest/hyper/)
- [`reqwest`](https://docs.rs/reqwest/0.11.23/reqwest/)
- [`jsonwebtoken`](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/)
- [`webrtc/examples/rtp-to-webrtc`](https://github.com/webrtc-rs/webrtc/tree/master/examples/examples/rtp-to-webrtc)
- [`v4l`](https://docs.rs/v4l/latest/v4l/)
