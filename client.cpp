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
//#include <algorithm>
#include <string>

#include <unistd.h>
#include <sys/mman.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <libcamera/libcamera.h>

using namespace libcamera;
using namespace std;

static int frameCount = 0;
static shared_ptr<Camera> camera;
static int fd;
static struct sockaddr_in serv_addr;

static void processRequest(Request *request);
static vector<Span<uint8_t>> mapBuffer(FrameBuffer *buffer);
static int writeFrame(uint8_t* data, size_t len, size_t cont);

static void requestHandler(Request *request) {
    if (request->status() == Request::RequestCancelled) {
        cout << "Request cancelled: " << request->status() << endl;
        return;
    }
    //cout << "Request status: " << request->status() << endl;
    processRequest(request);
    // Extremely temporary solution while we implement an event loop.
    frameCount++;
}

static void processRequest(Request *request) {
    for (auto const [stream, buffer] : request->buffers()) {
        //cout << "Buffer sequence: " << buffer->metadata().sequence << endl;
        vector<Span<uint8_t>> mappedPlanes = mapBuffer(buffer);
        assert(mappedPlanes.size() == buffer->planes().size());

        for (uint8_t i = 0; i < mappedPlanes.size(); i++) {
            Span<uint8_t> data = mappedPlanes[i];
            const auto len = min<unsigned int>(buffer->metadata().planes()[i].bytesused, data.size());
            cout << "Sending frame " << frameCount << " with length " << len << endl;
            writeFrame(data.data(), len, i);
        }
        
    }
    request->reuse(Request::ReuseBuffers);
    camera->queueRequest(request);
}

static vector<Span<uint8_t>> mapBuffer(FrameBuffer *buffer) {
    vector<Span<uint8_t>> planes;
    for (const auto &plane : buffer->planes()) {
        const int fd = plane.fd.get();
        const size_t len = max(
                static_cast<size_t>(lseek(fd, 0, SEEK_END)),
                static_cast<size_t>(plane.length + plane.offset)
        );
        void *addr = mmap(nullptr, len, PROT_READ, MAP_SHARED, fd, 0);
        if (addr != MAP_FAILED) {
            planes.emplace_back(static_cast<uint8_t *>(addr) + plane.offset, plane.length);
        }
    }
    return planes;
}

static int writeFrame(uint8_t* data, size_t len, size_t cont) {
    //if (cont == 0) {
    //    if (send(fd, "NEWFRAME", 9, 0) < 0) {
    //        cout << "Failed to send frame header" << endl;
    //        return -1;
    //    }
    //}
    if (send(fd, data, len, 0) < 0) {
        cout << "Failed to send frame data; frameCount: " << frameCount << endl;
        return -1;
    }
    return 0;
}

int main() {

    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) {
        cout << "Failed to create socket" << endl;
        return EXIT_FAILURE;
    }

    serv_addr.sin_family = AF_INET;
    serv_addr.sin_port = htons(8888);
    serv_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
    if (connect(fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
        cout << "Failed to connect to server" << endl;
        return EXIT_FAILURE;
    }


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
    camera = cm->get(cameraIds[0]);
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

    // Wait for 10 frames to be captured.
    // while (frameCount < 100) {}
    while (1) {}

    camera->stop();
    allocator->free(stream);
    delete allocator;
    camera->release();
    camera.reset();
    cm->stop();

    return EXIT_SUCCESS;
}
