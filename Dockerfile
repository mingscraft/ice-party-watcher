# Rust as the base image
FROM rust:1.64.0-alpine as builder

RUN apk add --no-cache musl-dev
RUN apk add --no-cache libressl-dev
RUN apk add --no-cache pkgconfig
RUN rustup target add armv7-unknown-linux-musleabihf
RUN apk add --no-cache binutils-arm-none-eabi gcc-arm-none-eabi 
RUN cargo install cross
RUN ln -s /usr/bin/arm-linux-gnueabihf-gcc /usr/bin/arm-linux-musleabihf-gcc

# Create a new empty shell project
RUN USER=root cargo new --bin ice-party-watch
WORKDIR /ice-party-watch

# Copy our manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./.cargo ./.cargo

# Build only the dependencies to cache them
RUN CC=arm-linux-gnueabihf-gcc cargo build --release --target armv7-unknown-linux-musleabihf
RUN rm ./src/*.rs
RUN rm ./target/release/deps/ice_party_watch*


# Copy the source code
COPY ./src ./src

# install openssl
RUN apk add --update openssl && \
    rm -rf /var/cache/apk/*

RUN CC=arm-linux-gnueabihf-gcc cargo build --release --target armv7-unknown-linux-musleabihf

FROM scratch
WORKDIR /ice-party-watch 
COPY --from=builder /ice-party-watch/target/armv7-unknown-linux-musleabihf/release/ice-party-watch /ice-party-watch/ice-party-watch

CMD ["./ice-party-watch"]
