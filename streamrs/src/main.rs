use axum::{
    body,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use opencv::{core::Vector, imgcodecs, prelude::*, videoio};
use std::{boxed::Box, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::{broadcast, Notify},
};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{
    self,
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

const CHANNEL_CAPACITY: usize = 16;

#[derive(Parser)]
struct Opts {
    #[arg(short('a'), long, default_value = "127.0.0.1")]
    http_host: String,
    #[arg(short('p'), long, default_value = "8080")]
    http_port: u16,
    #[arg(short, long, default_value = "12345")]
    cam_port: u16,
}

struct AppState {
    notify: Arc<Notify>,
    tx: Arc<broadcast::Sender<Box<[u8]>>>,
}

async fn multiplex_stream(
    notify: Arc<Notify>,
    tx: broadcast::Sender<Box<[u8]>>,
    port: u16,
) -> Result<(), opencv::Error> {
    notify.notified().await;
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
        if tx.receiver_count() <= 1 {
            #[cfg(debug_assertions)]
            println!("Channel is idle, waiting for notification.");
            notify.notified().await;
        }
        if let Err(e) = cap.read(&mut frame) {
            println!("Error reading frame: {}.", e);
        }
        println!("Frame: {:?}", frame.data_bytes().unwrap().get(0..8));
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
                        println!("\n{} current receivers.", rxs);
                    }
                    print!("Transmitter queue depth: {}\r", tx.len());
                }
            }
            Err(e) => println!("Error sending frame: {}.", e),
        }
    }
}

async fn feed_mjpeg(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    app_state.notify.notify_one();
    let rx = app_state.tx.subscribe();
    #[cfg(debug_assertions)]
    {
        println!("Channel is empty? {}.", app_state.tx.is_empty());
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
    println!(
        "Starting server on http://{}:{}",
        opts.http_host, opts.http_port
    );
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (tx, mut rx_drain) = broadcast::channel(CHANNEL_CAPACITY);
    let notify = Arc::new(Notify::new());
    let _ = tokio::spawn(multiplex_stream(notify.clone(), tx.clone(), opts.cam_port));
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
        .with_state(Arc::new(AppState {
            notify,
            tx: tx_state,
        }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
        );
    let listener = TcpListener::bind(format!("{}:{}", opts.http_host, opts.http_port))
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
