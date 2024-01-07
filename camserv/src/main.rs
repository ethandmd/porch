mod c2;
mod capture;
mod error;

use capture::{Capture, CaptureConfig};

#[tokio::main]
async fn main() -> Result<(), error::CamServError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    log::info!("Starting camserv...");
    let _c2 = c2::C2Listener;
    let conf = CaptureConfig::default();
    let cap = Capture::new(conf)?;
    cap.run().await?;
    Ok(())
}
