use axum::{
    body,
    extract::connect_info::ConnectInfo,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use futures::stream::Stream;
use lazy_static::lazy_static;
use opencv::{core::Vector, imgcodecs, prelude::*, videoio};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

#[derive(Parser)]
struct Opts {
    #[arg(short, long, default_value = "12345")]
    port: u16,
}

lazy_static! {
    static ref OPTS: Opts = Opts::parse();
}

struct StreamCapture {
    cap: videoio::VideoCapture,
}

impl StreamCapture {
    async fn new(port: u16) -> Result<Self, opencv::Error> {
        let cap = videoio::VideoCapture::from_file(format!(
                "udpsrc port={} caps=application/x-rtp ! rtph264depay ! h264parse ! decodebin ! videoconvert ! appsink",
                port).as_str(),
            videoio::CAP_GSTREAMER,
        )?;
        if !videoio::VideoCapture::is_opened(&cap)? {
            Err(opencv::Error::new(
                1,
                format!("Unable to open video stream @ udp:// {}", port),
            ))
        } else {
            Ok(Self { cap })
        }
    }
}

impl Stream for StreamCapture {
    type Item = Result<std::boxed::Box<[u8]>, opencv::Error>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = Vector::new();
        let mut frame = Mat::default();
        self.get_mut().cap.read(&mut frame).unwrap();
        if let Err(e) = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new()) {
            println!("Error: {}", e);
            return Poll::Ready(None);
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
        Poll::Ready(Some(Ok(bytes.into_boxed_slice())))
    }
}

// TODO: unwrap() on builder.
async fn feed_mjpeg(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    println!("User: {addr} connected.");
    if let Ok(stream) = StreamCapture::new(OPTS.port).await {
        Response::builder()
            .status(200)
            .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
            .body(body::Body::from_stream(stream))
            .unwrap()
        //body::Body::from_stream(stream)
    } else {
        Response::builder()
            .status(500)
            .body(body::Body::from("Unable to open video stream."))
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let app = Router::new()
        .fallback_service(ServeDir::new(assets).append_index_html_on_directories(true))
        .route("/feed/mjpeg", get(feed_mjpeg))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
