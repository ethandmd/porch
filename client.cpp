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

int main()
{
    std::unique_ptr<CameraManager> cm = std::make_unique<CameraManager>();
    cm->start();

    for (auto const &camera : cm->cameras()) {
        std::cout << "Found Camera with ID: " << camera->id() << std::endl;
        /*for (const auto &[id, info] : camera->controls()) {
		    std::cout << "Control: " << id->name() << ": " << info.toString() << std::endl;
	    }
        for (const auto &[key, value] : camera->properties()) {
	    	const ControlId *id = properties::properties.at(key);

		    std::cout << "Property: " << id->name() << " = " << value.toString() << std::endl;
	    }*/
        std::cout << "Camera model:" << *camera->properties().get(properties::Model) << std::endl;
        std::cout << std::endl;
    }

    cm->stop();
}
