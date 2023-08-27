## Porch camera client code

### Setup:
1. Install dependencies (assumes Ubuntu - Debian-ish env)
```
sudo apt install libcamera-dev
sudo apt install ninja-build
# for test_server.py
pip install opencv-python
pip install flask
```
3. Build and run:
```
git clone <this repo>
cd porch
porch~$ meson setup build
porch~$ ninja -C build # or `meson compile -C build`
porch~$ cd build
porch/build~$ ./porch
```

### [Libcamera API Guide](https://git.libcamera.org/libcamera/libcamera.git/tree/Documentation/guides/application-developer.rst)

### Relevent tf links:
1. TensorFlow 2 Detection Model Zoo
```
https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/tf2_detection_zoo.md
```
```
wget http://download.tensorflow.org/models/object_detection/tf2/20200711/<MODEL_NAME>.tar.gz
tar -xzvf path/to/<MODEL_NAME>.tar.gz -C destination_directory/
```
replace MODEL_NAME with desired model from tf2 model zoo. Define variables pipline_config, model_dir, and PATH_TO_LABELS with correct filepaths
