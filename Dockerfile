FROM rust:latest as builder
WORKDIR /app

RUN apt-get update && apt-get -y install cmake
ENV RUSTFLAGS="-C target-feature=+sse4.2"


COPY . .

RUN RUST_BACKTRACE=1 cargo build --release

# Run the binary
CMD ["/app/target/release/client_rs"]