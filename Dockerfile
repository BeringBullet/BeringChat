FROM rust:1.85-slim AS builder
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

RUN mkdir -p crates/server/src
RUN echo "fn main() {}" > crates/server/src/main.rs
RUN cargo build -p federated-server --release

COPY crates/server/src crates/server/src
RUN cargo build -p federated-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/federated-server /app/federated-server

ENV RUST_LOG=info
EXPOSE 8080
ENTRYPOINT ["/app/federated-server"]