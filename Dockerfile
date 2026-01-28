# Multi-stage build for ARM Hypervisor API Server
# Supports both x86_64 and ARM64 architectures

# Build stage
FROM --platform=$BUILDPLATFORM rust:latest as builder

# Install build dependencies for cross-compilation
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Set up build arguments for cross-compilation
ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install target triple based on target platform
RUN if [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
        rustup target add aarch64-unknown-linux-gnu; \
    elif [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        rustup target add x86_64-unknown-linux-gnu; \
    fi

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the API server in release mode
RUN if [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
        RUSTFLAGS="-C target-feature=+crt-static" cargo build --release \
        --bin api-server --target aarch64-unknown-linux-gnu 2>&1 && \
        cp target/aarch64-unknown-linux-gnu/release/api-server /tmp/api-server; \
    else \
        RUSTFLAGS="-C target-feature=+crt-static" cargo build --release \
        --bin api-server --target x86_64-unknown-linux-gnu 2>&1 && \
        cp target/x86_64-unknown-linux-gnu/release/api-server /tmp/api-server; \
    fi

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies and LXC
RUN apt-get update && apt-get install -y \
    ca-certificates \
    lxc \
    lxc-dev \
    libseccomp-dev \
    libcap-dev \
    apparmor \
    libc6 \
    libssl3 \
    curl \
    wget \
    iputils-ping \
    net-tools \
    iptables \
    dnsmasq \
    && rm -rf /var/lib/apt/lists/*

# Enable AppArmor (if running with CAP_SYS_ADMIN)
RUN echo "lxc.apparmor.profile = generated" >> /etc/lxc/default.conf || true

# Create non-root user for the API server
RUN useradd -m -s /bin/bash hypervisor

# Copy built binary from builder
COPY --from=builder /tmp/api-server /usr/local/bin/api-server
RUN chmod +x /usr/local/bin/api-server

# Create necessary directories
RUN mkdir -p \
    /etc/arm-hypervisor \
    /var/lib/lxc \
    /var/lib/hypervisor \
    /var/log/arm-hypervisor \
    && chown -R hypervisor:hypervisor /var/lib/hypervisor /var/log/arm-hypervisor

# Copy example configuration
COPY config.toml.example /etc/arm-hypervisor/config.toml.example

# Create default configuration for container environment
RUN cat > /etc/arm-hypervisor/config.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 8080
worker_threads = 4
keep_alive = 75
max_connections = 1000

[logging]
level = "info"
format = "json"

[security]
auth_enabled = false

[storage]
default_path = "/var/lib/hypervisor/storage"

[container]
lxc_path = "/var/lib/lxc"
default_template = "alpine"
EOF

# Set working directory
WORKDIR /var/lib/hypervisor

# Expose API port
EXPOSE 8080 8443

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Create startup script
RUN cat > /entrypoint.sh << 'EOF'
#!/bin/bash
set -e

# Initialize LXC if not already done
if [ ! -f "/var/lib/lxc/lxc-initialized" ]; then
    echo "Initializing LXC..."
    lxc-create -t none -n test 2>/dev/null && lxc-destroy test 2>/dev/null || true
    touch /var/lib/lxc/lxc-initialized
fi

# Start the API server
exec /usr/local/bin/api-server
EOF

RUN chmod +x /entrypoint.sh

# Use root to allow LXC operations (required for containers)
# In production, consider using capabilities or a privileged container
USER root

ENTRYPOINT ["/entrypoint.sh"]
