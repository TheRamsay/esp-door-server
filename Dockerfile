<<<<<<< HEAD
<<<<<<< HEAD
# Build Stage
FROM rust:latest as builder

RUN USER=root cargo new --bin axum-demo
WORKDIR ./server
COPY ./Cargo.toml ./Cargo.toml
# Build empty app with downloaded dependencies to produce a stable image layer for next build
RUN cargo build --release

# Build web app with own code
RUN rm src/*.rs
ADD . ./
RUN rm ./target/release/deps/server*
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

COPY --from=builder /axum-demo/target/release/server ${APP}/axum-demo

USER $APP_USER
WORKDIR ${APP}

CMD ["./server"]
=======
# ---------------------------------------------------
# 1 - Build Stage
#
# Use official rust image to for application build
# ---------------------------------------------------
FROM rust:latest as build
=======
FROM rust:1.65.0 as builder
>>>>>>> 7593c18c27c9da1705549a60afe165ff2fab9ce3

RUN USER=root cargo new --bin server
WORKDIR /server
COPY ./Cargo.toml ./Cargo.toml

RUN apt-get update && apt-get install -y curl libpq-dev build-essential

# Build empty app with downloaded dependencies to produce a stable image layer for next build
RUN cargo build --release

# Build web app with own code
RUN rm src/*.rs
COPY . ./
RUN rm ./target/release/deps/server*
RUN cargo build --release


FROM debian:buster-slim

RUN apt-get update && apt-get install -y curl libpq-dev build-essential

COPY --from=builder /server/target/release/server ./server

CMD ["./server"]
