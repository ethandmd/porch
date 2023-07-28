## Porch camera client code

### Setup:
1. Install `libcamera`
2. Build and run:
```
.. porch~$ meson setup build
.. porch~$ ninja -C build # or `meson compile -C build`
.. porch~$ ./porch
```

### Guide:

1. Install the `libcamera` library. Either use a package manager (e.g. `apt install libcamera-tools`) or build from source.

2. Initialize an instance of the `CameraManager` class:
```
// Only construct one CamMgr per process.
std::unique_ptr<CameraManager> cm = std::make_unique<CameraManager>();
cm->start();
```

3. Configure camera properties using the `CameraConfiguration` class:
```
std::string cameraId = cm->cameras()[0]->id();
camera = cm->get(cameraId);
camera->acquire();
std::unique_ptr<CameraConfiguration> config = camera->generateConfiguratoin( { StreamRole::ViewFinder } );
```

4. Capture images! After configuring the camera, you can begin capturing frames with:
```
std::unique_ptr<Request> request = camera->createRequest();
```
You will need to store captures in frame buffers either allocated by your app, or exported from the Camera by libcamera:
```
const std::vector<std::unique_ptr<FrameBuffer>> &buffers = allocator->buffers(stream);
std::vector<std::unique_ptr<Request>> requests;
```
Then, for each good request, you fetch and add the appropriate buffer to the request. Finally, you need to register an event
handler:
```
static void requestComplete(Request *request);
```
Libcamera uses a signal and slot method, so for every completed request signal from the Camera, the connected Slot is invoked.
However, the Slot is invoked in the CameraManager's thread, so you want to redirect request processing to your app's thread:

    - Signals are events 'emitted' by a class instance.
    - Slots are callbacks that can be 'connected' to a Signal.
```
static void processRequest(Request *request);

static void requestComplete(Request *request)
{
	if (request->status() == Request::RequestCancelled)
		return;
    //loop := event loop
	loop.callLater(std::bind(&processRequest, request));
}
```
Next, capture images by starting the camera, queue up requests, and in the demo case, they use an event loop to dispatch
events received from video devices, like buffer completions.

5. Clean up resources.
    - Stop the Camera
    - Free the stream used by the frame buffer allocator
    - Free the allocator
    - Release and reset the Camera
    - Stop the camera manager.
