## Porch camera client code

### Setup:
1. Install `libcamera`
   ```
   sudo apt install libcamera-dev
   sudo apt install ninja-build
   ```
3. Build and run:
```
.. porch~$ meson setup build
.. porch~$ ninja -C build # or `meson compile -C build`
.. porch~$ cd build
.. porch/build~$ ./porch
```

### [Guide](https://git.libcamera.org/libcamera/libcamera.git/tree/Documentation/guides/application-developer.rst)
