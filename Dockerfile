FROM rust:1.68-alpine as builder

RUN apk add --no-cache musl-dev
WORKDIR /usr/src
COPY . .

ARG TARGETPLATFORM
RUN case ${TARGETPLATFORM:-linux/amd64} in \
    "linux/amd64")   TARGET=x86_64-unknown-linux-musl;; \
    "linux/arm64")   TARGET=aarch64-unknown-linux-musl;; \
    "linux/arm/v7")  TARGET=armv7-unknown-linux-musleabi;; \
    *)                exit 1;; esac \
    && echo "TARGET=$TARGET" \
    && rustup target add ${TARGET}  \
    && cargo install --target ${TARGET} --path web

FROM scratch
COPY --from=builder /usr/local/cargo/bin/web .
COPY .env .
COPY cities500.txt .
USER 1000
CMD ["./web"]
