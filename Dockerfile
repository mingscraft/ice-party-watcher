# # Rust as the base image
# FROM rust:latest as builder
#  
# RUN apt update; apt upgrade -y 
# RUN apt install pkg-config -y
# RUN apt-get install libssl-dev -y
# RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross
#  
# RUN rustup target add aarch64-unknown-linux-gnu 
# RUN rustup toolchain install stable-aarch64-unknown-linux-gnu 
# 
# # Create a new empty shell project
# RUN USER=root cargo new --bin ice-party-watch
# WORKDIR /ice-party-watch
# 
# # Copy our manifests
# COPY ./Cargo.toml ./Cargo.toml
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./.cargo ./.cargo
# 
# ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
#     CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
#     CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
# 
# RUN PKG_CONFIG_SYSROOT_DIR=/
# 
# # Build only the dependencies to cache them
# RUN TARGET_CC=clang cargo build --release --target aarch64-unknown-linux-gnu
# RUN rm ./src/*.rs
# 
# 
# # Copy the source code
# COPY ./src ./src
# 
# RUN TARGET_CC=clang cargo build --release --target aarch64-unknown-linux-gnu
# 
# FROM scratch
# WORKDIR / 
# COPY --from=builder /ice-party-watch/target/aarch64-unknown-linux-gnu/release/ice-party-watch /ice-party-watch
# 
# CMD ["/ice-party-watch"]

FROM rust:1.61.0 as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new ice-party-watch 

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/ice-party-watch/

# Set the working directory
WORKDIR /usr/src/ice-party-watch

# This is a dummy build to get the dependencies cached.
RUN cargo build --release

# Now copy in the rest of the sources
COPY src /usr/src/ice-party-watch/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/ice-party-watch/src/main.rs

# This is the actual application build.
RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR / 
COPY --from=builder /usr/src/ice-party-watch/target/release/ice-party-watch /ice-party-watch
CMD ["/ice-party-watch"]
