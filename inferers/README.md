Example use (dog is outside in front of camera), assumes camera feed is sending data via udp to this host at port 12345/udp.

```
$ python3 model2onnx.py
$ cargo run yolov8n.onnx
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/inferepy yolov8n.onnx`
Setting up onnx runtime session with model: "yolov8n.onnx"
[ WARN:0] global ./modules/videoio/src/cap_gstreamer.cpp (1100) open OpenCV | GStreamer warning: Cannot query video position: status=1, value=6, duration=-1
Video capture opened with GSTREAMER.
Model input Input { name: "images", input_type: Float32, dimensions: [Some(1), Some(3), Some(640), Some(640)] }
Model output Output { name: "output0", output_type: Float32, dimensions: [Some(1), Some(84), Some(8400)] }
dog: 0.8384832
dog: 0.83205974
dog: 0.8281802
dog: 0.82058966
dog: 0.8150375

```
