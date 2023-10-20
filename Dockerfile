FROM rust:1.69-alpine as chef
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef --locked
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
ARG TARGETPLATFORM
COPY --from=planner /app/recipe.json recipe.json
# Cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN case ${TARGETPLATFORM:-linux/amd64} in \
    "linux/amd64")   TARGET=x86_64-unknown-linux-musl;; \
    "linux/arm64")   TARGET=aarch64-unknown-linux-musl;; \
    "linux/arm/v7")  TARGET=armv7-unknown-linux-musleabi;; \
    *)                exit 1;; esac \
    && echo "TARGET=$TARGET" \
    && rustup target add ${TARGET}  \
    && cargo build --locked --release --target ${TARGET}
RUN cargo install --locked --path web

FROM scratch
COPY --from=builder /usr/local/cargo/bin/web .
COPY cities.txt .
USER 1000
ENTRYPOINT ["./web"]
