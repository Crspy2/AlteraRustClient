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

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/altera_client /app/altera_client
RUN chmod +x /app/altera_client

# Run your binary
CMD ["/app/altera_clientt"]
