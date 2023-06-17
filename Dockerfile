FROM rust:latest as build

RUN USER=root cargo new --bin auth-server
WORKDIR /auth-server

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

RUN rm src/*.rs
COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /auth-server/target/release/auth-server .

CMD ["./auth-server"]