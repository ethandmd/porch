use crate::error::CamServError;
use chrono::Utc;
use futures::executor::LocalPool;
use futures::prelude::*;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use image;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

pub struct CaptureConfig {
    pub source: String,
    pub format: gst_video::VideoFormat,
    pub width: i32,
    pub height: i32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        CaptureConfig {
            source: "libcamerasrc".to_string(),
            format: gst_video::VideoFormat::Yuy2,
            width: 640,
            height: 480,
        }
    }
}

macro_rules! cb {
    ($sink:ident) => {
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |$sink| {
                let sample = $sink.pull_sample().map_err(|e| {
                    log::error!("Failed to pull sample: {}", e);
                    gst::FlowError::Eos
                })?;
                let buffer = sample.buffer().ok_or_else(|| {
                    log::error!("Failed to get buffer from sample");
                    gst::element_error!(
                        $sink,
                        gst::ResourceError::Failed,
                        ("Failed to get buffer from sample")
                    );
                    gst::FlowError::Error
                })?;
                let map = buffer.map_readable().map_err(|e| {
                    log::error!("Failed to map buffer readable: {}", e);
                    gst::element_error!(
                        $sink,
                        gst::ResourceError::Failed,
                        ("Failed to map buffer readable")
                    );
                    gst::FlowError::Error
                })?;
                let img = image::load_from_memory(map.as_slice()).map_err(|e| {
                    log::error!("Failed to load image from memory: {}", e);
                    gst::element_error!(
                        $sink,
                        gst::ResourceError::Failed,
                        ("Failed to load image from memory")
                    );
                    gst::FlowError::Error
                })?;
                log::debug!("{} x {}", img.width(), img.height());
                Ok(gst::FlowSuccess::Ok)
            })
            .build()
    };
}

pub struct Capture {
    pipeline: gst::Pipeline,
}

impl Capture {
    pub fn new(config: CaptureConfig) -> Result<Capture, CamServError> {
        gst::init().map_err(|e| {
            log::error!("Failed to initialize gstreamer: {}", e);
            CamServError::GstreamerInit
        })?;
        let pipeline = gst::Pipeline::default();
        let src = gst::ElementFactory::make(config.source.as_str())
            .build()
            .map_err(|e| {
                log::error!("Failed to create src: {}", e);
                CamServError::GstreamerInit
            })?;
        let caps = gst_video::VideoCapsBuilder::new()
            .format(config.format)
            .width(config.width)
            .height(config.height)
            .build();
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)
            .build()
            .map_err(|e| {
                log::error!("Failed to create capsfilter: {}", e);
                CamServError::GstreamerInit
            })?;
        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|e| {
                log::error!("Failed to create videoconvert: {}", e);
                CamServError::GstreamerInit
            })?;
        let queue = gst::ElementFactory::make("queue").build().map_err(|e| {
            log::error!("Failed to create queue: {}", e);
            CamServError::GstreamerInit
        })?;
        let jpeg = gst::ElementFactory::make("jpegenc").build().map_err(|e| {
            log::error!("Failed to create jpegenc: {}", e);
            CamServError::GstreamerInit
        })?;
        let queue2 = gst::ElementFactory::make("queue").build().map_err(|e| {
            log::error!("Failed to create queue2: {}", e);
            CamServError::GstreamerInit
        })?;
        let sink = gst_app::AppSink::builder().build();
        sink.set_callbacks(cb!(sink));
        pipeline
            .add_many([
                &src,
                &capsfilter,
                &videoconvert,
                &queue,
                &jpeg,
                &queue2,
                sink.upcast_ref(),
            ])
            .map_err(|e| {
                log::error!("Failed to add elements to pipeline: {}", e);
                CamServError::GstreamerInit
            })?;
        src.link(&capsfilter).map_err(|e| {
            log::error!("Failed to link src and capsfilter: {}", e);
            CamServError::GstreamerInit
        })?;
        capsfilter.link(&videoconvert).map_err(|e| {
            log::error!("Failed to link capsfilter and videoconvert: {}", e);
            CamServError::GstreamerInit
        })?;
        videoconvert.link(&queue).map_err(|e| {
            log::error!("Failed to link videoconvert and queue: {}", e);
            CamServError::GstreamerInit
        })?;
        queue.link(&jpeg).map_err(|e| {
            log::error!("Failed to link queue and jpeg: {}", e);
            CamServError::GstreamerInit
        })?;
        jpeg.link(&queue2).map_err(|e| {
            log::error!("Failed to link jpeg and queue2: {}", e);
            CamServError::GstreamerInit
        })?;
        queue2.link(&sink).map_err(|e| {
            log::error!("Failed to link queue2 and sink: {}", e);
            CamServError::GstreamerInit
        })?;
        #[cfg(debug_assertions)]
        {
            let dbg = pipeline.debug_to_dot_data(gst::DebugGraphDetails::all());
            let dot_fname = "pipeline.dot";
            log::debug!("Saving pipeline graph to dot file: {}", dot_fname);
            std::fs::write(dot_fname, dbg).map_err(|e| {
                log::error!("Failed to write dot file: {}", e);
                CamServError::GstreamerInit
            })?;
        }
        Ok(Capture { pipeline })
    }

    pub async fn run(&self) -> Result<(), CamServError> {
        log::info!("Setting up capture loop");
        let bus = self.pipeline.bus().ok_or(CamServError::CaptureLoop)?;
        self.pipeline.set_state(gst::State::Playing).map_err(|e| {
            log::error!("Failed to set pipeline to Playing: {}", e);
            CamServError::GstreamerPipeline
        })?;
        let mut pool = LocalPool::new();
        pool.run_until(capture_loop(bus)).map_err(|e| {
            log::error!("Failed to run capture loop: {}", e);
            CamServError::CaptureLoop
        })?;
        //tokio::spawn(capture_loop(bus)).await.map_err(|e| {
        //    log::error!("Failed to run capture loop: {}", e);
        //    CamServError::CaptureLoop
        //})??;
        log::info!("Cleaning up capture loop");
        self.pipeline.set_state(gst::State::Null).map_err(|e| {
            log::error!("Failed to set pipeline to Null: {}", e);
            CamServError::GstreamerPipeline
        })?;
        Ok(())
    }
}

pub async fn capture_loop(bus: gst::Bus) -> Result<(), CamServError> {
    let mut messages = bus.stream();
    while let Some(msg) = messages.next().await {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                log::info!("Capture loop: EOS");
                break;
            }
            MessageView::Error(e) => {
                log::error!(
                    "Capture loop: Error from {:?}: {}",
                    e.src().map(|s| s.path_string()),
                    e.error()
                );
                log::debug!("Debugging info: {:?}", e.debug());
                return Err(CamServError::CaptureLoop);
            }
            _ => {
                //println!("Capture loop: {:?}", msg);
            }
        }
    }
    Ok(())
}
async fn write_frame(img_bytes: Vec<u8>, t: f64) -> Result<(), CamServError> {
    let f = File::create(format!("{}_{}.jpg", Utc::now(), t))
        .await
        .map_err(|e| {
            log::error!("Failed to create file: {}", e);
            CamServError::WriteFrame
        })?;
    let mut writer = BufWriter::new(f);
    writer.write_all(img_bytes.as_slice()).await.map_err(|e| {
        log::error!("Failed to write frame to stdout: {}", e);
        CamServError::WriteFrame
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CaptureConfig {
        CaptureConfig {
            source: "videotestsrc".to_string(),
            format: gst_video::VideoFormat::Rgb,
            width: 640,
            height: 480,
        }
    }

    #[test]
    fn test_capture_init() {
        let capture = Capture::new(test_config());
        assert!(capture.is_ok());
    }
}
