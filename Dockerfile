###########
# Builder #
###########
FROM rust:1.68-slim as builder

# Preparation for cross-compilation
RUN apt update && apt-get install -y gcc-x86-64-linux-gnu
RUN rustup target add x86_64-unknown-linux-musl
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
WORKDIR /calar

# Copy only list of dependencies and make dummy main.rs
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src/
RUN echo 'fn main() { println!("You should not see this") }' > src/main.rs

# Build all dependecies without app itself, so they can be cached
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm -f target/x86_64-unknown-linux-musl/release/deps/calar*

# Copy source code itself
ADD src src

# Build the binary
RUN cargo build --target x86_64-unknown-linux-musl --release


###########
# Runtime #
###########
FROM --platform=linux/amd64 scratch AS runtime

# Copy the binary from build container
COPY --from=builder /calar/target/x86_64-unknown-linux-musl/release/calar /

# Run the server!
CMD ["/calar", "server"]

