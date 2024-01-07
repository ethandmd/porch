use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CamServError {
    CaptureLoop,
    GstreamerInit,
    GstreamerPipeline,
    WriteFrame,
}

impl fmt::Display for CamServError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CamServError::CaptureLoop => write!(f, "Capture loop error"),
            CamServError::GstreamerInit => write!(f, "Gstreamer init error"),
            CamServError::GstreamerPipeline => write!(f, "Gstreamer pipeline error"),
            CamServError::WriteFrame => write!(f, "Write frame error"),
        }
    }
}

impl Error for CamServError {}
