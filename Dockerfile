###################
##### Builder #####
###################
FROM rustlang/rust:nightly-slim as builder

WORKDIR /usr/src

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev

# Create blank project
RUN USER=root cargo new calar

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/calar/

WORKDIR /usr/src/calar

# This is a dummy build to get the dependencies cached.
RUN cargo +nightly build --release

# Now copy in the rest of the sources
COPY src /usr/src/calar/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/calar/src/main.rs

# This is the actual application build.
RUN cargo +nightly build --release

###################
##### Runtime #####
###################
FROM debian:buster-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev

# Copy application binary from builder image
COPY --from=builder /usr/src/calar/target/release/calar /usr/local/bin

# Run the application
CMD ["/usr/local/bin/calar", "server"]

