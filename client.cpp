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

int main()
{
    unique_ptr<CameraManager> cm = make_unique<CameraManager>();
    cm->start();

    if (cm->cameras().empty()) {
        cout << "No cameras found" << endl;
        return EXIT_FAILURE;
    }

    for (auto const &camera : cm->cameras()) {
        cout << "Found Camera with ID: " << camera->id() << endl;
        /*for (const auto &[id, info] : camera->controls()) {
		    cout << "Control: " << id->name() << ": " << info.toString() << endl;
	    }
        for (const auto &[key, value] : camera->properties()) {
	    	const ControlId *id = properties::properties.at(key);

		    cout << "Property: " << id->name() << " = " << value.toString() << endl;
	    }*/
        cout << "Camera model:" << *camera->properties().get(properties::Model) << endl;
        cout << endl;
    }

    cm->stop();
}
