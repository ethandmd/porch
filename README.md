# Porch

## Usage
Examples tested on raspberry pi 3 model B:
```
# Run this on source:
$ ./sender.sh -i libcamerasrc -o 192.168.1.10 -p 12345
...
# and on receiver:
cd ./streamrs
cargo run
```
```
# Run this on source
$ libcamera-vid -n -t 0 -o - | ./sender.sh -i fdsrc -o 192.168.1.10 -p 12345
...
# and on receiver
cd ./streamrs
cargo run
```
