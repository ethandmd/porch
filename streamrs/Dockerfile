FROM rust:latest as builder

WORKDIR /usr/src/streamrs
COPY . .
RUN apt-get update && apt-get dist-upgrade -y && apt-get autoremove -y --purge && apt-get -y autoclean
RUN apt-get install -y curl clang libclang-dev libopencv-dev && rm -rf /var/lib/lists/*
RUN cargo install --path . && cp /usr/local/cargo/bin/streamrs /usr/local/bin/streamrs
RUN apt-get install -y libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base \
gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x \
gstreamer1.0-alsa gstreamer1.0-gl

EXPOSE 12345
EXPOSE 8080
CMD ["streamrs", "-a", "0.0.0.0", "-p", "8080", "-c", "12345"]
