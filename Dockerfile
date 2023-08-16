# Use the official Ubuntu 22.04 LTS image as the base image
FROM rust:1.71-alpine

# Install required dependencies
RUN apk add --update \
    alpine-sdk \
    ffmpeg \
    youtube-dl \
    pkgconfig \
    cmake \
    openssl-dev \
    musl-dev \
    openssl

# Create a new directory for your application
WORKDIR /app

# Copy the Rust application source code to the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Command to run your application
CMD ["./target/release/yoitsus"]
