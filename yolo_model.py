from ultralytics import YOLO
from ultralytics.utils import ASSETS
from ultralytics.models.yolo.detect import DetectionPredictor
import cv2

model = YOLO("yolov8n.pt")

# Use the model
# model.train(data="coco128.yaml", epochs=3)  # train the model
# metrics = model.val()  # evaluate model performance on the validation set

# results = model("https://ultralytics.com/images/bus.jpg")  # predict on an image
results = model.predict(source="0", show=True)

# path = model.export(format="onnx")  # export the model to ONNX format

print(results)