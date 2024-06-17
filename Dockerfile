FROM rust:1.77-alpine3.19 as builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM alpine:3.19

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/registry-mirror-proxy .

EXPOSE 3000

CMD ["./registry-mirror-proxy"]