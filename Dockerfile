FROM rust:1.65.0 as builder

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