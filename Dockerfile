FROM rust:1.68-alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src
#RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add aarch64-unknown-linux-musl

COPY . .
RUN cargo build
RUN cargo install --target aarch64-unknown-linux-musl --path web

FROM scratch
COPY --from=builder /usr/local/cargo/bin/web .
COPY .env .
COPY cities.csv .
USER 1000
CMD ["./web"]
