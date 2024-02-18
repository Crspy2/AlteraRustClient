FROM rust:latest as builder

COPY . .

RUN apt-get update && apt-get -y install cmake
RUN RUST_BACKTRACE=1 cargo build --release

# Run the binary
CMD ["./target/release/client_rs"]