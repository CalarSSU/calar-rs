FROM rust:1.68-slim as builder
RUN apt update && apt-get install -y gcc-x86-64-linux-gnu
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
RUN rustup target add x86_64-unknown-linux-gnu
WORKDIR /usr/src/calar
COPY . .
RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM --platform=linux/amd64 debian:buster-slim AS runtime
COPY --from=builder /usr/src/calar/target/x86_64-unknown-linux-gnu/release/calar /usr/local/bin
CMD ["/usr/local/bin/calar", "server"]

