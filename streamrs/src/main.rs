use axum::{
    body,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use lazy_static::lazy_static;
use opencv::{core::Vector, imgcodecs, prelude::*, videoio};
use std::{boxed::Box, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{
    self,
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

#[derive(Parser)]
struct Opts {
    #[arg(short('a'), long, default_value = "127.0.0.1")]
    http_host: String,
    #[arg(short('p'), long, default_value = "8080")]
    http_port: u16,
    #[arg(short, long, default_value = "12345")]
    cam_port: u16,
}

lazy_static! {
    static ref OPTS: Opts = Opts::parse();
}

async fn multiplex_stream(
    tx: broadcast::Sender<Box<[u8]>>,
    port: u16,
) -> Result<(), opencv::Error> {
    let mut cap = videoio::VideoCapture::from_file(format!(
            "udpsrc port={} caps=application/x-rtp ! rtph264depay ! h264parse ! decodebin ! videoconvert ! appsink",
            port).as_str(),
        videoio::CAP_GSTREAMER,
    )?;
    let _ = cap.is_opened()?;
    println!("Video capture opened with {}.", cap.get_backend_name()?);
    let mut frame = Mat::default();
    let mut buf = Vector::new();
    #[cfg(debug_assertions)]
    let mut rxs = 0;
    loop {
        if tx.receiver_count() == 0 {
            continue;
        }
        if let Err(e) = cap.read(&mut frame) {
            println!("Error reading frame: {}.", e);
        }
        buf.clear();
        if let Err(e) = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new()) {
            println!("Error encoding frame: {}", e);
        }
        let response = format!(
            "{}{}{}{}{}",
            "--frame\r\n",
            "Content-Type: image/jpeg\r\n",
            "Content-Length: ",
            buf.len(),
            "\r\n\r\n",
        );
        let mut bytes = response.into_bytes();
        bytes.append(&mut buf.as_slice().to_vec());
        bytes.append(&mut b"\r\n".to_vec());
        match tx.send(bytes.into()) {
            Ok(n) => {
                #[cfg(debug_assertions)]
                {
                    if n != rxs {
                        rxs = n;
                        println!("{} current receivers.", rxs);
                    }
                    print!("Transmitter queue depth: {}\r", tx.len());
                }
            }
            Err(e) => println!("Error sending frame: {}.", e),
        }
    }
}

async fn feed_mjpeg(State(tx): State<Arc<broadcast::Sender<Box<[u8]>>>>) -> impl IntoResponse {
    let rx = tx.subscribe();
    #[cfg(debug_assertions)]
    {
        println!("Channel is empty? {}.", tx.is_empty());
        println!("Receiver queue depth: {}.", rx.len());
    }
    Response::builder()
        .status(200)
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .body(body::Body::from_stream(BroadcastStream::new(rx)))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (tx, mut rx_drain) = broadcast::channel(16);
    let _ = tokio::spawn(multiplex_stream(tx.clone(), opts.cam_port));
    let tx_state = Arc::new(tx.clone());
    //TODO: issue with receiving frames in feed_mjpeg without this workaround.
    tokio::spawn(async move {
        loop {
            let _ = rx_drain.recv().await;
        }
    });
    let app = Router::new()
        .fallback_service(ServeDir::new(assets).append_index_html_on_directories(true))
        .route("/feed/mjpeg", get(feed_mjpeg))
        .with_state(tx_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
        );
    let listener = TcpListener::bind(format!("{}:{}", OPTS.http_host, OPTS.http_port))
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
