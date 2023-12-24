use clap::{Parser, ValueEnum};
use glib::MainLoop;
//use gstreamer::prelude::*;
//use gstreamer::{ElementFactory, Pipeline, State};
use gstreamer_rtsp_server::prelude::*;
use gstreamer_rtsp_server::{
    gst, gst::DeviceMonitor, RTSPAddressPool, RTSPClient, RTSPMediaFactory, RTSPServer,
};

#[derive(Parser)]
#[command(author = "ethandmd")]
#[command(about = "Stream video from libcamerasrc or v4l2src using rtsp server.")]
struct Cli {
    #[arg(
        short,
        long,
        required = true,
        help = "Stream source.",// (libcamerasrc, libcamera-vid, v4l2src)",
        //hide_possible_values = true
    )]
    source: Sources,
    #[arg(short = 'p', long)]
    source_params: Option<String>,
    #[arg(short, long)]
    encoder: Option<String>,
    #[arg(short, long)]
    list_devices: bool,
}

#[derive(Copy, Clone, ValueEnum)]
enum Sources {
    Fdsrc,
    Libcamerasrc,
    V4l2src,
}

struct LaunchCommand(String);

impl LaunchCommand {
    fn new(source: Sources, params: Option<String>, encoder: Option<String>) -> Self {
        let orr = String::from("");
        let enc = String::from("x264enc");
        match source {
            Sources::Fdsrc => {
                Self::from(["fdsrc", &params.unwrap_or(orr), &encoder.unwrap_or(enc)])
            }
            Sources::Libcamerasrc => Self::from([
                "libcamerasrc",
                &params.unwrap_or(orr),
                &encoder.unwrap_or(enc),
            ]),
            Sources::V4l2src => {
                Self::from(["v4l2src", &params.unwrap_or(orr), &encoder.unwrap_or(enc)])
            }
        }
    }
}

impl From<[&str; 3]> for LaunchCommand {
    fn from(source: [&str; 3]) -> Self {
        match source[0] {
            "libcamerasrc" => {
                // Consider format=I420 for libcamerasrc
                LaunchCommand(format!(
                    "( libcamerasrc {} ! \
                    capsfilter caps=video/x-raw,format=NV12 ! \
                    videoconvert ! \
                    {} tune=zerolatency ! \
                    queue ! \
                    rtph264pay name=pay0 pt=96 )",
                    source[1], source[2]
                ))
            }
            "fdsrc" => LaunchCommand(format!(
                "( fdsrc {} ! \
                    h264parse ! \
                    rtph264pay name=pay0 pt=96 )",
                source[1]
            )),
            "v4l2src" => LaunchCommand(format!(
                "( v4l2src {} ! \
                    video/x-raw,width=640,height=480 ! \
                    videoconvert ! \
                    {} tune=zerolatency ! \
                    queue ! \
                    rtph264pay name=pay0 pt=96 )",
                source[1], source[2]
            )),
            _ => panic!("Invalid source"),
        }
    }
}

fn enumerate_devices() {
    let mon = DeviceMonitor::new();
    mon.add_filter(Some("Video/Source"), None);
    mon.start().expect("Failed to start device monitor");
    for dev in mon.devices() {
        println!("Device: {}", dev.display_name());
        println!("\tClass: {}", dev.device_class());
        //print!("\tCaps:");
        //if let Some(caps) = dev.caps() {
        //    println!(" {}", caps);
        //}
    }
    mon.stop();
}

fn serve(source: Sources, params: Option<String>, encoder: Option<String>) {
    let launch = LaunchCommand::new(source, params, encoder);
    println!("Launch command: {}", &launch.0);
    let main_loop = MainLoop::new(None, false);
    let server = RTSPServer::new();
    let mounts = server.mount_points().expect("Could not get mount points.");
    let factory = RTSPMediaFactory::new();
    let addr_pool = RTSPAddressPool::new();
    let _ = addr_pool
        .add_range("224.3.0.1", "224.3.0.255", 5000, 5256, 16)
        .expect("Failed to add address pool.");
    factory.set_address_pool(Some(&addr_pool));
    //factory.set_protocols(RTSPLowerTrans::UDP_MCAST);
    factory.set_launch(&launch.0);
    factory.set_shared(true);
    mounts.add_factory("/cam", factory);
    server.connect_client_connected(|_server: &RTSPServer, _client: &RTSPClient| {
        println!("Client connected");
    });
    let id = server
        .attach(None)
        .expect("Failed to attach to main context.");
    println!(
        "Streaming at rtsp://{}:{}/cam",
        server.address().expect("Server ain't listening."),
        server.bound_port()
    );
    main_loop.run();
    id.remove();
}

fn main() {
    gst::init().expect("Failed to initialize gstreamer");
    let args = Cli::parse();
    if args.list_devices {
        enumerate_devices();
        return;
    }
    // Initialize gstreamer
    serve(args.source, args.source_params, args.encoder);
}
