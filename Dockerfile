FROM rust:latest as builder

WORKDIR /app

# Install system dependencies if any
# For example, to ensure you have the necessary compilers and tools for SIMD
# and any other project-specific system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --bin altera_client

FROM ubuntu:latest

COPY --from=builder /app/target/release/altera_client /app/target/release/altera_client
RUN chmod +x /app/target/release/altera_client

ENV RUST_BACKTRACE=full

# Run your binary
CMD ["/app/target/release/altera_client"]
