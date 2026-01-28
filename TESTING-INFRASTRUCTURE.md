# ARM Hypervisor Testing Infrastructure

This directory contains the testing infrastructure for the ARM Hypervisor project, enabling rapid development and validation in virtual machines and containerized environments.

## Quick Start

### Linux/macOS

```bash
# Make script executable
chmod +x quickstart.sh

# Run quick start
./quickstart.sh

# View logs
./quickstart.sh logs

# Stop services
./quickstart.sh stop
```

### Windows (PowerShell)

```powershell
# Run quick start
.\quickstart.bat

# For subsequent operations, use docker-compose directly:
docker-compose logs -f api-server
docker-compose down
```

## Files Overview

### Docker Configuration

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build for x86_64 and ARM64 architectures |
| `.dockerignore` | Excludes unnecessary files from Docker build context |
| `docker-compose.yml` | Orchestrates API server and optional cluster nodes |

### Scripts

| File | Purpose |
|------|---------|
| `quickstart.sh` | Linux/macOS quick start script |
| `quickstart.bat` | Windows quick start script |

### Configuration

| File | Purpose |
|------|---------|
| `config.toml.example` | Example configuration (included in Docker image) |
| `config.vm-test.toml` | Optimized configuration for VM testing |

### CI/CD

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | Rust compilation, testing, and linting |
| `.github/workflows/docker.yml` | Docker image building for multiple platforms |

### Documentation

| File | Purpose |
|------|---------|
| `TESTING.md` | Comprehensive testing guide |
| `DEPLOYMENT.md` | Deployment instructions |
| `TESTING-INFRASTRUCTURE.md` | This file |

## Testing Methods

### 1. Local Unit Tests (No Docker)

For quick feedback during development:

```bash
cargo test --lib
cargo test --lib -p container-manager
```

**Pros**: Fast, no dependencies  
**Cons**: Doesn't test system integration

### 2. Docker Container

For consistent, reproducible testing:

```bash
docker-compose up -d
docker-compose exec api-server cargo test
```

**Pros**: Consistent environment, includes LXC  
**Cons**: Requires Docker, slightly slower startup

### 3. Virtual Machine

For realistic production-like testing:

```bash
# SSH into VM
ssh ubuntu@vm-ip

# Build and test
cd arm-hypervisor-main
cargo build --release
cargo test --workspace
```

**Pros**: Real system environment, actual LXC operations  
**Cons**: Requires separate VM, more setup time

### 4. CI/CD Pipeline (GitHub Actions)

Automated testing on every push:

```bash
# View in GitHub UI or:
gh run list
gh run view <run-id>
```

**Pros**: Automated, multi-platform testing (x86_64 + ARM64)  
**Cons**: Only runs on push, limited to GitHub's runners

## Docker Compose Services

### api-server (Primary)

The main ARM Hypervisor API server:

```bash
# Logs
docker-compose logs -f api-server

# Access container shell
docker-compose exec api-server bash

# Run specific test
docker-compose exec api-server cargo test test_container_creation_request_validation
```

### test-node (Optional Cluster Node)

For cluster testing, enable with profile:

```bash
# Start with test node
docker-compose --profile cluster up -d

# Check cluster status
curl http://localhost:8080/api/cluster/status
```

## Volumes

Docker Compose creates persistent volumes for:

- `lxc_data`: LXC container filesystem
- `hypervisor_data`: Application data
- `hypervisor_logs`: Application logs

To clean up:

```bash
docker-compose down -v  # Removes volumes
```

## Networking

### Port Mappings

- **8080** → API Server (HTTP)
- **8443** → API Server (HTTPS, if configured)

### Internal Network

Services communicate via `hypervisor_net` bridge network:

```bash
# View network
docker network inspect arm-hypervisor-main_hypervisor_net
```

## Building Multi-Platform Docker Images

### Prerequisites

```bash
docker buildx create --name multiarch --driver docker-container
docker buildx use multiarch
```

### Build for Multiple Platforms

```bash
# Build for both amd64 and arm64 (requires buildx)
docker buildx build --platform linux/amd64,linux/arm64 -t arm-hypervisor:multi .

# Build and push to registry
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/user/arm-hypervisor:latest \
  --push .
```

## VM Setup Instructions

### Quick VM Creation (AWS EC2)

```bash
# Launch ARM64 instance
aws ec2 run-instances \
  --image-id ami-0<ubuntu-22-04-arm64> \
  --instance-type t4g.medium \
  --key-name your-keypair \
  --security-group-ids sg-xxxxxxxx

# SSH in and setup
ssh -i key.pem ubuntu@ec2-xx-xx-xx-xx.compute-1.amazonaws.com

# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker ubuntu

# Clone and test
git clone <repo-url> && cd arm-hypervisor-main
./quickstart.sh
```

### Raspberry Pi Setup

```bash
# 1. Flash 64-bit Raspberry Pi OS using Raspberry Pi Imager
# 2. Enable SSH and set password
# 3. Boot and connect

ssh pi@raspberry.local

# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker pi

# Run tests
./quickstart.sh
```

## Troubleshooting

### Docker won't start on Windows

**Issue**: "Docker Desktop is not running"  
**Solution**: Start Docker Desktop application

### Permission denied when running containers

**Issue**: User not in docker group  
**Solution**: 
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### Docker build fails with platform error

**Issue**: "platform linux/arm64 not available"  
**Solution**: Install and use buildx
```bash
docker buildx create --name mybuilder --use
```

### Tests fail with "LXC command failed"

**Issue**: LXC not installed in container  
**Solution**: Dockerfile automatically installs LXC; rebuild:
```bash
docker-compose down
docker-compose up --build -d
```

### Docker Compose port 8080 already in use

**Issue**: Port conflict  
**Solution**: 
```bash
# Use different port
docker run -p 8888:8080 -d arm-hypervisor:latest

# Or stop other service
docker-compose down
```

## Performance Notes

- First Docker build: ~3-5 minutes
- Subsequent builds (cached): ~30 seconds  
- Container startup: ~5-10 seconds
- Unit test execution: ~10-15 seconds
- Full test suite: ~2-3 minutes
- ARM64 emulation (cross-platform): ~5-10 minutes

## Next Steps

1. **Run quick start**: `./quickstart.sh`
2. **Read TESTING.md**: Detailed testing strategies
3. **Check DEPLOYMENT.md**: Production deployment
4. **Join cluster**: Use `--profile cluster` for multi-node testing
5. **Monitor CI/CD**: Watch GitHub Actions for automated tests

## Support

For issues or questions:

1. Check [TESTING.md](TESTING.md) for comprehensive guide
2. Review [Troubleshooting](#troubleshooting) section
3. Check Docker/Docker Compose documentation
4. Create an issue on the project repository
