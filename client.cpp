/*
 * Client component of the Porch project.
 * This component is responsible for:
 * 1. Detecting available cameras connected to the host it runs on.
 * 2. Capturing and detecting frames from the cameras.
 * 3. Sending the frames to the server component.
 *
 */

#include <iostream>
#include <memory>

#include <libcamera/libcamera.h>
using namespace libcamera;
using namespace std;

static void requestHandler(Request *request) {
    if (request->status() == Request::RequestCancelled) {
        cout << "Request cancelled: " << request->status() << endl;
        return;
    }
    cout << "Request complete: " << request->status() << endl;
}

int main()
{
    vector<string> cameraIds;
    unique_ptr<CameraManager> cm = make_unique<CameraManager>();
    cm->start();

    if (cm->cameras().empty()) {
        cout << "No cameras found" << endl;
        return EXIT_FAILURE;
    }
    
    // Enumerate all cameras.
    for (auto const &cam : cm->cameras()) {
        auto cameraId = cam->id();
        cameraIds.push_back(cameraId);
        cout << "Found Camera with ID: " << cameraId << endl;
        cout << "Camera model: " << *cam->properties().get(properties::Model) << endl;
        cout << endl;
    }
    
    // Grab our favorite camera.
    shared_ptr<Camera> camera = cm->get(cameraIds[0]);
    camera->acquire();
    
    // Set the camera + stream configuration.
    unique_ptr<CameraConfiguration> config = camera->generateConfiguration( { StreamRole::Viewfinder } );
    StreamConfiguration &streamConfig = config->at(0);
    config->validate();
    cout << "Viewfinder configuration: " << streamConfig.toString() << endl;
    camera->configure(config.get());

    // Allocate frame buffers for the stream.
    FrameBufferAllocator *allocator = new FrameBufferAllocator(camera);
    for (StreamConfiguration &cfg : *config) {
        if (allocator->allocate(cfg.stream())< 0) {
            cout << "Failed to allocate buffers for stream " << cfg.toString() << endl;
            return EXIT_FAILURE;
        }
    }
    
    // Allocate requests using the allocated frame buffers.
    Stream *stream = streamConfig.stream();
    const vector<unique_ptr<FrameBuffer>> &buffers = allocator->buffers(stream);
    vector<unique_ptr<Request>> requests;

    for (auto const &buf : buffers) {
        unique_ptr<Request> request = camera->createRequest();
        if (!request) {
            cout << "Can't create request for camera: " << camera->id() << endl;
            return EXIT_FAILURE;
        }
        if (request->addBuffer(stream, buf.get()) < 0) {
            cout << "Can't set buffer for request" << endl;
            return EXIT_FAILURE;
        }
        requests.push_back(move(request));
    }
    
    // Register the slot function to receive the camera signals.
    camera->requestCompleted.connect(requestHandler);
    
    // Start the camera and queue the requests.
    if (camera->start() < 0) {
        cout << "Can't start camera: " << camera->id() << endl;
        return EXIT_FAILURE;
    }

    for (auto const &request : requests) {
        if (camera->queueRequest(request.get()) < 0) {
            cout << "Can't queue request" << endl;
            return EXIT_FAILURE;
        }
    }
    

    camera->stop();
    allocator->free(stream);
    delete allocator;
    camera->release();
    cm->stop();

    return EXIT_SUCCESS;
}
