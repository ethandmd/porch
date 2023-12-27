use opencv::{highgui, prelude::*, videoio, Result};

fn main() -> Result<()> {
    highgui::named_window("video", highgui::WINDOW_FULLSCREEN)?;
    let mut cam = videoio::VideoCapture::from_file(
        "udpsrc port=12345 caps=application/x-rtp ! rtph264depay ! h264parse ! decodebin ! videoconvert ! appsink",
        videoio::CAP_GSTREAMER,
    )?;
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to receive capture!");
    }
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.size()?.width > 0 {
            highgui::imshow("video", &mut frame)?;
        }
        highgui::wait_key(1)?;
    }
}
