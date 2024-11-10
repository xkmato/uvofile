FROM rust:latest AS builder
RUN USER=root RUST_BACKTRACE=full cargo new --bin uvofile
WORKDIR /uvofile

COPY Cargo.toml Cargo.lock ./

COPY src ./src
RUN apt-get update && apt-get install -y libpq-dev ca-certificates
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /uvofile/target/release/uvofile /usr/local/bin/uvofile

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

EXPOSE 8080

CMD ["uvofile"]