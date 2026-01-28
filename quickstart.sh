#!/bin/bash
# ARM Hypervisor Quick Start Script
# Sets up testing environment and runs the project in Docker

set -e

# Colors for output
RED='\\033[0;31m'
GREEN='\\033[0;32m'
YELLOW='\\033[1;33m'
NC='\\033[0m' # No Color

# Functions
print_status() {
    echo -e \"${GREEN}[✓]${NC} $1\"
}

print_info() {
    echo -e \"${YELLOW}[*]${NC} $1\"
}

print_error() {
    echo -e \"${RED}[✗]${NC} $1\"
}

# Check prerequisites
check_prerequisites() {
    print_info \"Checking prerequisites...\"
    
    if ! command -v docker &> /dev/null; then
        print_error \"Docker is not installed\"
        echo \"Install Docker from https://docs.docker.com/get-docker/\"
        exit 1
    fi
    print_status \"Docker found: $(docker --version)\"
    
    if ! command -v docker-compose &> /dev/null; then
        print_error \"Docker Compose is not installed\"
        echo \"Install Docker Compose from https://docs.docker.com/compose/install/\"
        exit 1
    fi
    print_status \"Docker Compose found: $(docker-compose --version)\"
}

# Build Docker image
build_image() {
    print_info \"Building Docker image...\"
    docker build -t arm-hypervisor:latest .
    print_status \"Docker image built successfully\"
}

# Start services
start_services() {
    print_info \"Starting Docker Compose services...\"
    docker-compose up -d
    print_status \"Services started\"
    
    print_info \"Waiting for API server to be ready...\"
    sleep 5
    
    # Check health
    if curl -f http://localhost:8080/health &> /dev/null 2>&1; then
        print_status \"API server is healthy\"
    else
        print_info \"Health check endpoint not yet available, but container is running\"
    fi
}

# Display endpoints
show_endpoints() {
    echo \"\"
    echo -e \"${GREEN}═══════════════════════════════════════${NC}\"
    echo -e \"${GREEN}ARM Hypervisor is ready for testing!${NC}\"
    echo -e \"${GREEN}═══════════════════════════════════════${NC}\"
    echo \"\"
    echo \"Endpoint Information:\"
    echo \"  API Server:  http://localhost:8080\"
    echo \"  Logs:        docker-compose logs -f api-server\"
    echo \"\"
    echo \"Test URLs:\"
    echo \"  Health:      curl http://localhost:8080/health\"
    echo \"  Containers:  curl http://localhost:8080/api/containers\"
    echo \"  Cluster:     curl http://localhost:8080/api/cluster/status\"
    echo \"\"
    echo \"Docker Commands:\"
    echo \"  Stop:        docker-compose down\"
    echo \"  Logs:        docker-compose logs -f api-server\"
    echo \"  Shell:       docker-compose exec api-server /bin/bash\"
    echo \"  Run tests:   docker-compose exec api-server cargo test\"
    echo \"\"
    echo \"Documentation:\"
    echo \"  Testing:     See TESTING.md\"
    echo \"  Deployment:  See DEPLOYMENT.md\"
    echo \"\"
}

# Main flow
main() {
    echo \"\"
    echo -e \"${GREEN}╔═══════════════════════════════════════╗${NC}\"
    echo -e \"${GREEN}║  ARM Hypervisor Quick Start Script    ║${NC}\"
    echo -e \"${GREEN}╚═══════════════════════════════════════╝${NC}\"
    echo \"\"
    
    # Parse arguments
    if [[ \"$1\" == \"stop\" ]]; then
        print_info \"Stopping services...\"
        docker-compose down
        print_status \"Services stopped\"
        exit 0
    fi
    
    if [[ \"$1\" == \"logs\" ]]; then
        docker-compose logs -f api-server
        exit 0
    fi
    
    if [[ \"$1\" == \"test\" ]]; then
        print_info \"Running tests inside container...\"
        docker-compose exec api-server cargo test --workspace
        exit 0
    fi
    
    if [[ \"$1\" == \"shell\" ]]; then
        docker-compose exec api-server /bin/bash
        exit 0
    fi
    
    if [[ \"$1\" == \"build-only\" ]]; then
        check_prerequisites
        build_image
        exit 0
    fi
    
    # Default: build and start
    check_prerequisites
    build_image
    start_services
    show_endpoints
}

# Run main function
main \"$@\"
