FROM alpine:latest as builder

RUN apk update && apk --no-cache --update add clang lld curl build-base protobuf

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH "$PATH:/root/.cargo/bin"

RUN mkdir ~/src

WORKDIR /root/src

COPY . .

RUN RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=lld" cargo build --release

FROM alpine:latest

WORKDIR /opt/cache-server

COPY --from=builder /root/src/target/release/cache-server ./

CMD ["/opt/cache-server/cache-server"]