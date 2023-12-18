use std::env;
use glib::MainLoop;
use gstreamer_rtsp_server::gst;
use gstreamer_rtsp_server::prelude::*;

fn serve() {
    let launch = env::args().skip(1).fold(String::new(), |a, b| format!("{a} {b}")); 
    let main_loop = MainLoop::new(None, false);
    let server = gstreamer_rtsp_server::RTSPServer::new();
    let mounts = server.mount_points().expect("Could not get mount points.");
    let factory = gstreamer_rtsp_server::RTSPMediaFactory::new();
    println!("Launch options: {}", &launch);
    factory.set_launch(&launch);
    factory.set_shared(true);
    mounts.add_factory("/cam", factory);
    let id = server.attach(None).expect("Failed to attach to main context.");
    println!("Streaming at rtsp://192.168.1.9:{}/cam", server.bound_port());
    main_loop.run();
    id.remove();
}

fn main() {
    gst::init().expect("Failed to init gstreamer.");
    serve();
}
