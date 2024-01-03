use image::GenericImageView;
use ndarray::{Array, Axis};
use opencv::{
    prelude::*,
    videoio::{VideoCapture, CAP_GSTREAMER},
};
use std::{env, path::PathBuf};

const NUM_INTER_THREADS: i16 = 4;

#[rustfmt::skip]
const YOLOV8_CLASS_LABELS: [&str; 80] = [
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat", "traffic light",
	"fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog", "horse", "sheep", "cow", "elephant",
	"bear", "zebra", "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard",
	"sports ball", "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket", "bottle",
	"wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange", "broccoli",
	"carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch", "potted plant", "bed", "dining table", "toilet",
	"tv", "laptop", "mouse", "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink", "refrigerator",
	"book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush"
];

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let model_file_path = PathBuf::from(env::args().nth(1).expect("Missing model file path"));
    println!(
        "Setting up onnx runtime session with model: {:?}",
        model_file_path
    );
    let ort_env = ort::Environment::builder()
        .with_name("inferers")
        .with_log_level(ort::LoggingLevel::Verbose)
        .with_execution_providers([ort::ExecutionProvider::CPU(
            ort::execution_providers::CPUExecutionProviderOptions::default(),
        )])
        .build()?
        .into_arc();

    let session = ort::SessionBuilder::new(&ort_env)?
        .with_optimization_level(ort::GraphOptimizationLevel::Level1)?
        .with_inter_threads(NUM_INTER_THREADS)?
        .with_model_from_file(model_file_path)?;

    let mut cap = VideoCapture::from_file(
            "udpsrc port=12345 caps=application/x-rtp ! rtph264depay ! h264parse ! decodebin ! videoconvert ! appsink",
        CAP_GSTREAMER,
    )?;
    let _ = cap.is_opened()?;
    println!("Video capture opened with {}.", cap.get_backend_name()?);

    #[cfg(debug_assertions)]
    for i in session.inputs.iter() {
        println!("Model input {:?}", i);
    }

    #[cfg(debug_assertions)]
    for i in session.outputs.iter() {
        println!("Model output {:?}", i);
    }

    let mut frame = Mat::default();
    cap.read(&mut frame)?;
    let mut buf = opencv::core::Vector::new();
    opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &opencv::types::VectorOfi32::new())?;
    let img = image::load_from_memory(&buf.as_slice())?;
    let mut input_tensor = Array::zeros((1, 3, 640, 640));
    for (x, y, rgb) in img.pixels() {
        let x = x as usize;
        let y = y as usize;
        let [r, g, b, _] = rgb.0;
        input_tensor[[0, 0, y, x]] = (r as f32) / 255.0;
        input_tensor[[0, 1, y, x]] = (g as f32) / 255.0;
        input_tensor[[0, 2, y, x]] = (b as f32) / 255.0;
    }
    let outputs = session.run(vec![ort::Value::from_array(
        session.allocator(),
        &ndarray::CowArray::from(input_tensor.into_dyn()),
    )?])?;

    let outs = outputs
        .get(0)
        .unwrap()
        .try_extract::<f32>()
        .unwrap()
        .view()
        .t()
        .into_owned();
    let mut classes = Vec::new();
    for row in outs.axis_iter(Axis(0)) {
        let (class_id, prob) = row
            .iter()
            .skip(4)
            .enumerate()
            .map(|(idx, val)| (idx, *val))
            .reduce(|ax, row| if ax.1 > row.1 { ax } else { row })
            .unwrap();
        let label = YOLOV8_CLASS_LABELS[class_id];
        //println!("{}: {}", label, prob);
        classes.push((label, prob));
    }
    // Get top N classes by probability.
    classes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    classes.iter().take(5).for_each(|(label, prob)| {
        println!("{}: {}", label, prob);
    });

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Error: {}", e);
    }
}
