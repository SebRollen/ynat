FROM rust:1.91 as builder

# Make a fake Rust app to keep a cached layer of compiled crates
RUN USER=root cargo new app
WORKDIR /usr/src/app
COPY . .
# Needs at least a main.rs file with a main function
RUN mkdir src && echo "fn main(){}" > src/main.rs
# Will build all dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --bin ynat-auth-server --features server

# Copy the rest
COPY . .
# Build (install) the actual binaries
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path ./ynat-auth --bin ynat-auth-server --features server

# Runtime image
FROM debian:13-slim

# Run as "app" user
RUN useradd -ms /bin/bash app
# Add in ssl certificates
RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

USER app
WORKDIR /app
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/ynat-auth-server /app/ynat-auth-server

CMD ./ynat-auth-server
