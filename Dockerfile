# Rust as the base image
FROM rust:latest as builder
 
RUN apt update; apt upgrade -y 
RUN apt install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross
 
RUN rustup target add armv7-unknown-linux-gnueabihf 
RUN rustup toolchain install stable-armv7-unknown-linux-gnueabihf 

# Create a new empty shell project
RUN USER=root cargo new --bin ice-party-watch
WORKDIR /ice-party-watch

# Copy our manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./.cargo ./.cargo

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
RUN SET PKG_CONFIG_SYSROOT_DIR=/

# Build only the dependencies to cache them
RUN cargo build --release --target armv7-unknown-linux-gnueabihf
RUN rm ./src/*.rs
RUN rm ./target/release/deps/ice_party_watch*


# Copy the source code
COPY ./src ./src

RUN cargo build --release --target armv7-unknown-linux-gnueabihf

FROM scratch
WORKDIR /ice-party-watch 
COPY --from=builder /ice-party-watch/target/armv7-unknown-linux-musleabihf/release/ice-party-watch /ice-party-watch/ice-party-watch

CMD ["./ice-party-watch"]
