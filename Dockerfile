# Rust as the base image
FROM rust:1.64.0-alpine3.15 as builder

RUN apk add --no-cache musl-dev
RUN apk add --no-cache libressl-dev
RUN apk add --no-cache pkgconfig

# Create a new empty shell project
RUN USER=root cargo new --bin ice-party-watch
WORKDIR /ice-party-watch

# Copy our manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm ./src/*.rs
RUN rm ./target/release/deps/ice_party_watch*


# Copy the source code
COPY ./src ./src

# install openssl
RUN apk add --update openssl && \
    rm -rf /var/cache/apk/*

RUN cargo build --release

# The final base image
FROM debian:buster-slim

FROM scratch
WORKDIR /ice-party-watch 
COPY --from=builder /ice-party-watch/target/release/ice-party-watch /ice-party-watch/ice-party-watch

CMD ["./ice-party-watch"]
