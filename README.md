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
