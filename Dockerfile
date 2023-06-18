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