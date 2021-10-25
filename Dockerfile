FROM rust:latest as builder

WORKDIR /usr/src/cache-server

COPY . .

RUN rustup component add rustfmt
RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /opt/cache-server

COPY --from=builder /usr/src/cache-server/target/release/cache-server ./

CMD ["/opt/cache-server/cache-server"]