FROM rustlang/rust:nightly-slim as builder
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
WORKDIR /usr/src/calar
COPY . .
RUN cargo +nightly build --release

FROM debian:buster-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends openssl
COPY --from=builder /usr/src/calar/target/release/calar /usr/local/bin
CMD ["/usr/local/bin/calar", "server"]

