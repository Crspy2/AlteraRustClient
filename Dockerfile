FROM rust:latest as builder
WORKDIR /app/discord

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This trick will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Now that the dependencies are built, copy your source code
COPY ./src ./src


# Build your application
RUN touch src/main.rs
RUN cargo build --release


FROM ubuntu:latest


RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/discord/target/release/app /app/discord/target/release/app

# Set the binary as the entrypoint of the container
ENTRYPOINT ["/app/discord/target/release/app"]