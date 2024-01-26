import sys
import threading
import numpy as np
from PyQt5 import QtCore, QtWidgets, QtGui
import cv2
from ultralytics import YOLO

class VideoWindow(QtWidgets.QWidget):

    # Define a signal
    update_frame_signal = QtCore.pyqtSignal(np.ndarray)

    def __init__(self, video_source):
        super().__init__()
        self.video_source = video_source
        self.current_frame = None
        self.initUI()
        self.update_frame_signal.connect(self.update_frame)

    def initUI(self):
        self.setGeometry(100, 100, 800, 600)
        self.setWindowTitle('Video Stream')
        self.show()

    def paintEvent(self, event):
        painter = QtGui.QPainter(self)
        if self.current_frame is not None:
            image = QtGui.QImage(self.current_frame, self.current_frame.shape[1], self.current_frame.shape[0], 
                                 self.current_frame.strides[0], QtGui.QImage.Format_RGB888)
            painter.drawImage(0, 0, image)

#    @QtCore.pyqtSlot(np.ndarray)
#     def update_frame(self, frame):
#         self.current_frame = frame
#         self.repaint()
            
    @QtCore.pyqtSlot(np.ndarray)
    def update_frame(self, frame):
        # Resize the frame to fit the window, maintaining aspect ratio
        aspect_ratio = frame.shape[1] / frame.shape[0]
        new_width = int(self.height() * aspect_ratio)
        resized_frame = cv2.resize(frame, (new_width, self.height()))

        # Center the frame in the window
        if new_width < self.width():
            x_offset = (self.width() - new_width) // 2
            self.current_frame = np.zeros((self.height(), self.width(), 3), dtype=np.uint8)
            self.current_frame[:, x_offset:x_offset+new_width] = resized_frame
        else:
            self.current_frame = resized_frame

        self.repaint()

#################

def run_tracker_in_thread(filename, model, file_index, video_window):
    video = cv2.VideoCapture(filename)
    while True:
        ret, frame = video.read()
        if not ret:
            break

        results = model.track(frame, persist=True)
        res_plotted = results[0].plot()

        # Emit the signal with the frame
        video_window.update_frame_signal.emit(res_plotted)

        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    video.release()

###################

def main():
    app = QtWidgets.QApplication(sys.argv)

    # IP camera URL(s)
    ip_camera_url1 = "http://195.196.36.242/mjpg/video.mjpg"
    ip_camera_url2 = "http://173.198.10.174//mjpg/video.mjpg"
    ip_camera_url3 = "http://217.171.212.63/mjpg/video.mjpg"

    # Create VideoWindow instances
    video_window1 = VideoWindow(ip_camera_url1)
    video_window2 = VideoWindow(ip_camera_url2)
    video_window3 = VideoWindow(ip_camera_url3)

    # Load the models
    model1 = YOLO('yolov8n.pt')
    model2 = YOLO('yolov8n-seg.pt')
    model3 = YOLO('yolov8n-seg.pt')

    # Create and start tracker threads
    tracker_thread1 = threading.Thread(target=run_tracker_in_thread, args=(ip_camera_url1, model1, 1, video_window1), daemon=True)
    tracker_thread2 = threading.Thread(target=run_tracker_in_thread, args=(ip_camera_url2, model2, 2, video_window2), daemon=True)
    tracker_thread3 = threading.Thread(target=run_tracker_in_thread, args=(ip_camera_url3, model3, 3, video_window3), daemon=True)
    
    tracker_thread1.start()
    tracker_thread2.start()
    tracker_thread3.start()

    sys.exit(app.exec_())

if __name__ == '__main__':
    main()

