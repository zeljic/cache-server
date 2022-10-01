FROM rust:latest as builder

RUN apt update && apt install -y protobuf-compiler; protoc --version

WORKDIR /usr/src/cache-server

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /opt/cache-server

COPY --from=builder /usr/src/cache-server/target/release/cache-server ./

CMD ["/opt/cache-server/cache-server"]