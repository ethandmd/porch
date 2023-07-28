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

    for (auto const &cam : cm->cameras()) {
        auto cameraId = cam->id();
        cameraIds.push_back(cameraId);
        cout << "Found Camera with ID: " << cameraId << endl;
        cout << "Camera model: " << *cam->properties().get(properties::Model) << endl;
        cout << endl;
    }
    // Grab our favorite camera.
    shared_ptr<Camera> camera = cm->get(cameraIds[1]);
    camera->acquire();
    
    unique_ptr<CameraConfiguration> config = camera->generateConfiguration( { StreamRole::Viewfinder } );
    StreamConfiguration &streamConfig = config->at(0);
    config->validate();
    cout << "Viewfinder configuration: " << streamConfig.toString() << endl;
    camera->configure(config.get());


    FrameBufferAllocator *allocator = new FrameBufferAllocator(camera);
    for (StreamConfiguration &cfg : *config) {
        if (allocator->allocate(cfg.stream())< 0) {
            cout << "Failed to allocate buffers for stream " << cfg.toString() << endl;
            return EXIT_FAILURE;
        }
    }

    Stream *stream = streamConfig.stream();
    const vector<unique_ptr<FrameBuffer>> &buffers = allocator->buffers(stream);
    vector<unique_ptr<Request>> requests;

    camera->requestCompleted.connect(requestHandler);
    buffers[0];

    camera->stop();
    allocator->free(stream);
    delete allocator;
    camera->release();
    cm->stop();

    return EXIT_SUCCESS;
}
