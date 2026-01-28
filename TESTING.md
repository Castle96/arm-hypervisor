# ARM Hypervisor Testing Guide

This guide provides comprehensive instructions for setting up, running, and validating tests for the ARM Hypervisor project in virtual machines and containerized environments.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Testing Strategies](#testing-strategies)
3. [Unit Tests](#unit-tests)
4. [Integration Tests](#integration-tests)
5. [Docker Testing](#docker-testing)
6. [Virtual Machine Setup](#virtual-machine-setup)
7. [Cluster Testing](#cluster-testing)
8. [CI/CD Pipeline](#cicd-pipeline)
9. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Cargo**: Comes with Rust
- **Docker** (for containerized testing): [Install Docker](https://docs.docker.com/get-docker/)
- **Docker Compose** (optional, for multi-node testing): [Install Docker Compose](https://docs.docker.com/compose/install/)

### Running Tests Locally (No Docker)

```bash
# Run all unit tests
cargo test --lib

# Run all tests with output
cargo test -- --nocapture

# Run tests in a specific crate
cargo test -p container-manager
cargo test -p network
cargo test -p storage
```

### Running Tests with Docker

```bash
# Build and start the API server in Docker
docker-compose up -d

# Check logs
docker-compose logs -f api-server

# Stop services
docker-compose down
```

## Testing Strategies

The project uses two complementary testing strategies:

### Strategy 1: Mocked Integration Tests (Recommended for CI/CD)

**Purpose**: Fast, reliable testing without system dependencies  
**When to use**: GitHub Actions, automated testing, quick feedback loops

- ✅ No LXC/system tool dependencies
- ✅ Runs in any environment (CI/CD, containers, Windows)
- ✅ Fast execution (~5-10 seconds)
- ✅ Deterministic results
- ❌ Doesn't test actual system integration

**Implementation**:
Uses the `mockall` crate to mock `ContainerManager`, `BridgeManager`, and storage managers.

See [integration tests](#integration-tests) for setup details.

### Strategy 2: Full Integration Tests (For VM/Container Environments)

**Purpose**: Test actual LXC container operations and networking  
**When to use**: VM testing, production validation, pre-deployment verification

- ✅ Tests real LXC operations
- ✅ Validates actual network configuration
- ✅ Comprehensive end-to-end testing
- ❌ Requires privileged container or VM with LXC
- ❌ Slower execution (~30-60 seconds per test)
- ❌ Environmental dependencies

**Prerequisites**:
- Ubuntu 20.04+ or Raspberry Pi OS
- LXC installed (`apt-get install lxc`)
- Kernel with LXC support (most modern kernels)
- AppArmor enabled (or disabled for testing)

## Unit Tests

Unit tests validate individual functions and modules without external dependencies.

### Running Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run unit tests for a specific crate
cargo test -p api-server --lib
cargo test -p container-manager --lib
cargo test -p network --lib
cargo test -p storage --lib

# Run a specific test
cargo test -p container-manager test_container_name_validation -- --nocapture

# Run tests matching a pattern
cargo test container_name

# Run with backtrace on failure
RUST_BACKTRACE=1 cargo test --lib
```

### Unit Test Coverage

**api-server/src/config.rs**:
- Configuration loading and validation
- TOML/YAML parsing
- Environment variable overrides

**api-server/src/middleware.rs**:
- Security headers injection
- CORS handling
- Request logging

**container-manager/src/lib.rs**:
- Container name validation (lowercase, hyphens, no dots)
- Container state parsing (running, stopped, starting, etc.)
- Configuration structure validation

**network/src/lib.rs**:
- Bridge name validation (format and length)
- IP address validation (IPv4/IPv6 CIDR notation)
- Network interface validation

**storage/src/lib.rs**:
- Storage pool name validation
- Storage path validation (must be absolute, not /tmp)
- Storage type validation

## Integration Tests

Integration tests verify multiple components working together. The crate now supports both mocked and full integration testing.

### Mocked Integration Tests (CI/CD Safe)

These tests use the `mockall` crate to avoid system dependencies.

```bash
# Run mocked integration tests only
cargo test --test '*' --features mocked

# Run all tests including mocked integration tests
cargo test

# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

**Features**:
- Mock implementations for LXC command execution
- Simulated container lifecycle operations
- Fake network bridge creation
- In-memory storage pool management

### Full Integration Tests (VM/Container Only)

These tests actually invoke LXC and system commands. They're skipped in CI by default.

```bash
# Run full integration tests on a machine with LXC
cargo test --features integration-tests -- --nocapture

# Run only container integration tests
cargo test -p container-manager -- --ignored

# Run with environment isolation
LXC_TEST_PREFIX=test_run_ cargo test -- --ignored
```

**Requirements**:
- LXC installed and configured
- `/var/lib/lxc` directory writable
- Linux kernel with cgroup support
- AppArmor or SELinux configured (or disabled)

**Test Isolation**:
- Each test creates containers with `test_` prefix
- Tests clean up containers after completion
- Can run tests in parallel with `--test-threads=4`

## Docker Testing

### Building Docker Image

```bash
# Build for current platform
docker build -t arm-hypervisor:latest .

# Build for specific platform
docker build --platform linux/arm64 -t arm-hypervisor:arm64 .
docker build --platform linux/amd64 -t arm-hypervisor:amd64 .

# Build with BuildKit (faster, parallel layers)
docker buildx build --platform linux/amd64,linux/arm64 -t arm-hypervisor:multi .
```

### Running Single Container

```bash
# Run with default config
docker run -d --name hypervisor \
  -p 8080:8080 \
  --privileged \
  arm-hypervisor:latest

# View logs
docker logs -f hypervisor

# Stop container
docker stop hypervisor
docker rm hypervisor
```

### Running with Docker Compose

```bash
# Start single API server
docker-compose up -d api-server

# Start API server + test node cluster
docker-compose --profile cluster up -d

# View logs
docker-compose logs -f api-server

# Run tests inside container
docker-compose exec api-server cargo test

# Stop all services
docker-compose down

# Clean up volumes
docker-compose down -v
```

### Docker Network Testing

```bash
# Check container networking
docker exec arm-hypervisor-api ip addr show
docker exec arm-hypervisor-api ip route show

# Test health check
curl http://localhost:8080/health

# List API endpoints
curl http://localhost:8080/api/containers
```

## Virtual Machine Setup

### Option 1: Local Hypervisor (KVM/VirtualBox)

#### Raspberry Pi Emulation (QEMU)

```bash
# Install QEMU for ARM64 emulation
apt-get install qemu-system-arm qemu-efi binfmt-support qemu-user-static

# Run QEMU VM with Docker
docker run -it --rm \
  --platform linux/arm64 \
  ubuntu:22.04 bash

# Inside the container, install and test
apt-get update
apt-get install -y docker.io curl
docker run arm-hypervisor:arm64
```

#### Ubuntu VM (VirtualBox/KVM)

1. **Create Ubuntu 22.04 VM**:
   - Allocate 4+ GB RAM
   - 20+ GB disk space
   - Enable nested virtualization if available

2. **Install Docker**:
   ```bash
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh
   ```

3. **Clone repository and test**:
   ```bash
   git clone <repo-url>
   cd arm-hypervisor-main
   docker-compose up -d
   ```

### Option 2: Cloud VMs (AWS, GCP, Azure)

#### AWS EC2 - ARM64 (Graviton2)

```bash
# Launch t4g.medium instance with Ubuntu 22.04
aws ec2 run-instances \
  --image-id ami-0<ubuntu-22-04-arm64> \
  --instance-type t4g.medium \
  --key-name your-key-pair

# SSH into instance
ssh -i your-key.pem ubuntu@<instance-ip>

# Install Docker and run tests
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker ubuntu
git clone <repo-url> && cd arm-hypervisor-main
docker-compose up
```

#### GCP - Compute Engine (ARM64)

```bash
gcloud compute instances create arm-hypervisor-test \
  --machine-type=t2a-standard-2 \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --zone=us-central1-a
```

### Option 3: Physical Raspberry Pi

#### Hardware Requirements

- **Raspberry Pi 4B or later** (4GB+ RAM)
- **64-bit Raspberry Pi OS**
- **High-speed microSD card** (Class 3 U3+)
- **USB-C power supply** (5V 3A+)

#### Setup

```bash
# 1. Flash Raspberry Pi OS (64-bit) using Raspberry Pi Imager
# 2. Enable SSH and set password
# 3. Boot and connect

# 4. SSH into Pi
ssh pi@raspberry.local

# 5. Update system
sudo apt-get update && sudo apt-get upgrade -y

# 6. Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker pi

# 7. Install Docker Compose
sudo apt-get install -y docker-compose

# 8. Clone and test
git clone <repo-url> && cd arm-hypervisor-main
docker-compose up
```

## Cluster Testing

### Testing Multi-Node Setup

```bash
# Start cluster with 1 API server + 1 test node
docker-compose --profile cluster up -d

# Verify both services are healthy
docker-compose ps

# Check cluster status
curl http://localhost:8080/api/cluster/status

# View node information
curl http://localhost:8080/api/nodes

# Stop cluster
docker-compose --profile cluster down
```

### Manual Multi-Node Testing

```bash
# Terminal 1: Start API server on node1
docker run -d -p 8080:8080 \
  -e NODE_NAME=node1 \
  --name node1 arm-hypervisor:latest

# Terminal 2: Start API server on node2
docker run -d -p 8081:8080 \
  -e NODE_NAME=node2 \
  -e PRIMARY_NODE=localhost:8080 \
  --name node2 arm-hypervisor:latest

# Join node2 to cluster
curl -X POST http://localhost:8081/api/cluster/join \
  -H "Content-Type: application/json" \
  -d '{"primary_node": "localhost:8080", "node_name": "node2"}'

# Check cluster
curl http://localhost:8080/api/cluster/status
```

## CI/CD Pipeline

### GitHub Actions Workflow

The project includes automated testing via `.github/workflows/ci.yml`:

**Stages**:
1. **Clippy (Linting)**: Code quality checks
2. **Rustfmt (Formatting)**: Code style verification
3. **Tests**: Run unit and mocked integration tests
4. **Security Audit**: Vulnerability scanning
5. **Docker Build**: Multi-platform image building

**Triggers**:
- Push to main branch
- Pull requests
- Tag pushes (releases)

**To view results**:

```bash
# GitHub Actions tab in repo
# or via CLI
gh run list
gh run view <run-id>
```

### Running Local CI Tests

```bash
# Run all CI checks locally
./scripts/ci-check.sh

# Or manually:
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo audit
```

## Troubleshooting

### Tests Fail with "LXC command failed"

**Cause**: LXC not installed or not in PATH  
**Solution**: Install LXC or run mocked tests

```bash
# Install LXC
sudo apt-get install lxc lxc-dev

# Run mocked tests instead
cargo test --features mocked
```

### Container Fails to Start in Docker

**Cause**: Privileged mode not enabled  
**Solution**: Use `--privileged` flag

```bash
docker run -it --privileged arm-hypervisor:latest
```

### "Permission denied" when running LXC tests

**Cause**: User doesn't have LXC permissions  
**Solution**: Add user to lxc group

```bash
sudo usermod -aG lxc $USER
newgrp lxc
```

### Docker Compose fails with "unknown instruction"

**Cause**: Docker Compose version too old  
**Solution**: Update Docker Compose

```bash
sudo apt-get install --only-upgrade docker-compose
# or
curl -L "https://github.com/docker/compose/releases/download/v2.x.x/docker-compose-$(uname -s)-$(uname -m)" \
  -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose
```

### Tests timeout in CI

**Cause**: System under heavy load or insufficient resources  
**Solution**: 
- Increase timeout values
- Reduce parallel test threads
- Check GitHub Actions runner availability

```bash
# Reduce parallelism
cargo test -- --test-threads=1

# Add explicit timeout
timeout 300 cargo test
```

### ARM64 Docker builds fail

**Cause**: Missing QEMU user static or Docker buildx  
**Solution**: Install buildx

```bash
docker buildx create --name multiarch --driver docker-container
docker buildx use multiarch
docker buildx build --platform linux/amd64,linux/arm64 .
```

## Performance Benchmarks

Expected test execution times (on modern hardware):

| Test Suite | Duration | Notes |
|-----------|----------|-------|
| Unit tests | ~5-10s | All platforms |
| Mocked integration tests | ~15-20s | All platforms |
| Full integration tests | ~2-5 min | Requires LXC + Linux |
| Docker build (single platform) | ~2-3 min | First build, cached: ~10s |
| Docker build (multi-platform) | ~5-10 min | First build, slow on emulation |
| Full CI pipeline | ~10-15 min | GitHub Actions |

## Next Steps

1. **Set up local testing**: Follow [Quick Start](#quick-start)
2. **Run Docker tests**: See [Docker Testing](#docker-testing)
3. **Deploy to VM**: Choose a VM option from [Virtual Machine Setup](#virtual-machine-setup)
4. **Monitor CI/CD**: Check [CI/CD Pipeline](#cicd-pipeline) for automated validation

## Additional Resources

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [LXC Documentation](https://linuxcontainers.org/lxc/getting-started/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Raspberry Pi Documentation](https://www.raspberrypi.com/documentation/)
