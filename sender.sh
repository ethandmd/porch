#!/bin/bash
# Example usage (tested on raspberry pi 3 model B):
#	$ ./stream.sh -i libcamerasrc -o 192.168.1.10 -p 12345
#	$ libcamera-vid -n -t 0 -o - | ./stream.sh -i fdsrc -o 192.168.1.10 -p 12345

# Capture input source, destination arguments:
# -i: input source
# -o: output host
# -p: output port
while getopts i:o:p:d: flag
do
	case "${flag}" in
		i) input=${OPTARG};;
		o) output=${OPTARG};;
		p) port=${OPTARG};;
		d) device=${OPTARG};;
	esac
done

# Check if input is provided. If not, exit.
if [ -z "$input" ]
then
	echo "Please provide input and output arguments"
	exit
fi

# Check if output is provided, if not, set host as 127.0.0.1 and port as 8554.
if [ -z "$output" ] && [ -z "$port" ]
then
	output="127.0.0.1"
	port="12345"
fi

# Check if gstreamer is installed on the system. If not, install.
if ! [ -x "$(command -v gst-launch-1.0)" ]; then
  echo 'Info: gstreamer is not installed. Installing' >&2
	# Check if script is run as root. If not, exit.
	if [ "$EUID" -ne 0 ]
	  then echo "Please run as root"
	  exit
	fi
  apt update
  apt install gstreamer1.0-tools \
	  gstreamer1.0-plugins-base \
	  gstreamer1.0-plugins-good \
	  gstreamer1.0-plugins-bad \
	  gstreamer1.0-plugins-ugly
fi

# Echo configuration and run gst-launch command.
echo "Input: $input"
echo "Output: $output:$port"

# Match input source on v4l2, fdsrc, or libcamerasrc and run the appropriate gst-launch command.
if [[ $input == *"v4l2"* ]]; then
	# If device is not provided, set device as /dev/video0.
	if [ -z "$device" ]
	then
		device="/dev/video0"
	fi
	launch="gst-launch-1.0 v4l2src device=$device"
	launch+=" ! video/x-raw,framerate=30/1"
	launch+=" ! videoconvert"
	launch+=" ! x264enc tune=zerolatency"
	launch+=" ! queue"
	launch+=" ! rtph264pay"
	launch+=" ! udpsink host=$output port=$port"
	elif [[ $input == *"fdsrc"* ]]; then
	#gst-launch-1.0 fdsrc fd=0 ! udpsink host=192.168.1.10 port=12345
	launch="gst-launch-1.0 fdsrc fd=0 ! h264parse ! rtph264pay config-interval=1 ! udpsink host=$output port=$port"
elif [[ $input == *"libcamerasrc"* ]]; then
	launch="gst-launch-1.0 libcamerasrc"
	launch+=" ! capsfilter caps=video/x-raw,format=NV12 "
	launch+=" ! videoconvert"
	launch+=" ! x264enc tune=zerolatency"
	launch+=" ! queue"
	launch+=" ! rtph264pay"
	launch+=" ! udpsink host=$output port=$port"
else
	echo "Input source not supported"
fi

echo "Running: $launch"

# Run gst-launch command, if source is fdsrc, redirect stdin to launched process.
if [[ $input == *"fdsrc"* ]]; then
	$launch <&0
else
	$launch
fi
