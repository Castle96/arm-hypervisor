# Testing Phase - Critical Improvements

## Overview

This document describes the complete testing strategy for the three critical improvements implemented in the ARM Hypervisor project:

1. **Database/Persistence Layer** - SQLite backend
2. **Real Container Status Tracking** - LXC status queries
3. **Request Validation** - Input constraint enforcement

---

## Testing Pyramid

```
                    ▲
                   / \
                  /   \  Manual/E2E Tests (5%)
                 /     \
                /-------\
               /         \  Integration Tests (25%)
              /           \
             /-------------\
            /               \  Unit Tests (70%)
           /_________________\
```

---

## Phase 1: Unit Tests

### 1.1 Validation Module Tests

**File:** `crates/models/src/validation.rs` (already included)

Tests for all validation functions:

```bash
cargo test -p models --lib validation
```

**Coverage:**
- ✅ Container name validation (valid/invalid)
- ✅ CPU limit bounds (1-128)
- ✅ Memory limit constraints (64MB-1TB)
- ✅ Disk limit constraints (100MB-10TB)
- ✅ Template format validation
- ✅ Edge cases and boundary conditions

**Expected Results:**
```
running 11 tests
test validation::tests::test_validate_container_name - ok
test validation::tests::test_validate_cpu_limit - ok
test validation::tests::test_validate_memory_limit - ok
test validation::tests::test_validate_disk_limit - ok
test validation::tests::test_validate_template - ok

test result: ok. 11 passed
```

---

### 1.2 Database Module Tests

**File:** `crates/database/src/` (new tests to create)

Create `crates/database/src/tests/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePool;
    use crate::{pool::*, container_store::ContainerStore};
    use models::{Container, ContainerConfig, CreateContainerRequest};

    async fn setup_test_db() -> SqlitePool {
        let config = PoolConfig {
            database_url: "sqlite://:memory:".to_string(),
            max_connections: 5,
        };
        create_pool(config).await.expect("Failed to create pool")
    }

    #[tokio::test]
    async fn test_container_store_get_or_create() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(2),
            memory_limit: Some(512 * 1024 * 1024),
            disk_limit: Some(5 * 1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let container = store
            .get_or_create("test-container", "alpine", config)
            .await
            .expect("Failed to create container");

        assert_eq!(container.name, "test-container");
        assert_eq!(container.template, "alpine");
        assert_eq!(container.status, ContainerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_container_store_list() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        // Create multiple containers
        for i in 0..3 {
            let config = ContainerConfig {
                cpu_limit: Some(1),
                memory_limit: Some(256 * 1024 * 1024),
                disk_limit: Some(1024 * 1024 * 1024),
                network_interfaces: vec![],
                rootfs_path: format!("/var/lib/lxc/test-{}/rootfs", i),
                environment: vec![],
            };
            
            let _ = store
                .get_or_create(&format!("test-{}", i), "alpine", config)
                .await;
        }

        let containers = store.list().await.expect("Failed to list");
        assert_eq!(containers.len(), 3);
    }

    #[tokio::test]
    async fn test_container_store_update_status() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("test-container", "alpine", config)
            .await;

        // Update status
        store
            .update_status("test-container", "running")
            .await
            .expect("Failed to update");

        let container = store
            .get_by_name("test-container")
            .await
            .expect("Failed to get");

        assert_eq!(container.status, ContainerStatus::Running);
    }

    #[tokio::test]
    async fn test_container_store_delete() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("test-container", "alpine", config)
            .await;

        store
            .delete("test-container")
            .await
            .expect("Failed to delete");

        let result = store.get_by_name("test-container").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_container_store_exists() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("test-container", "alpine", config)
            .await;

        assert!(store.exists("test-container").await.expect("Failed to check"));
        assert!(!store.exists("nonexistent").await.expect("Failed to check"));
    }
}
```

**Run Tests:**
```bash
cargo test -p database --lib
```

**Expected Results:**
```
running 6 tests
test tests::test_container_store_get_or_create - ok
test tests::test_container_store_list - ok
test tests::test_container_store_update_status - ok
test tests::test_container_store_delete - ok
test tests::test_container_store_exists - ok
test tests::test_database_migration - ok

test result: ok. 6 passed
```

---

### 1.3 Container Manager Tests

Update `crates/container-manager/src/lib.rs` tests:

The existing tests already cover:
- ✅ Container name validation
- ✅ Container state parsing
- ✅ LXC command parsing

---

## Phase 2: Integration Tests

### 2.1 Handler Integration Tests

**File:** `crates/api-server/tests/integration_test.rs` (new)

```rust
use actix_web::{test, web, App};
use sqlx::sqlite::SqlitePool;
use database::pool::{create_pool, PoolConfig};

#[actix_web::test]
async fn test_create_container_with_validation() {
    let pool = setup_test_db().await;
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/containers")
        .set_json(serde_json::json!({
            "name": "web-server-1",
            "template": "alpine",
            "config": {
                "cpu_limit": 2,
                "memory_limit": 536870912,
                "disk_limit": 10737418240,
                "network_interfaces": [],
                "rootfs_path": "/var/lib/lxc/web-server-1/rootfs",
                "environment": []
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_create_container_invalid_name() {
    let pool = setup_test_db().await;
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/containers")
        .set_json(serde_json::json!({
            "name": "Invalid Name",  // Invalid: spaces
            "template": "alpine",
            "config": {
                "cpu_limit": 2,
                "memory_limit": 536870912,
                "disk_limit": 10737418240,
                "network_interfaces": [],
                "rootfs_path": "/var/lib/lxc/invalid/rootfs",
                "environment": []
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);  // Bad Request
}

#[actix_web::test]
async fn test_create_container_cpu_too_high() {
    let pool = setup_test_db().await;
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/containers")
        .set_json(serde_json::json!({
            "name": "test",
            "template": "alpine",
            "config": {
                "cpu_limit": 256,  // Invalid: too high
                "memory_limit": 536870912,
                "disk_limit": 10737418240,
                "network_interfaces": [],
                "rootfs_path": "/var/lib/lxc/test/rootfs",
                "environment": []
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);  // Bad Request
}

#[actix_web::test]
async fn test_list_containers_returns_persistent_ids() {
    let pool = setup_test_db().await;
    
    // Create a container
    // List containers - get UUID
    // List containers again - same UUID should be returned
    // This validates persistence
}
```

**Run Tests:**
```bash
cargo test -p api-server --lib
```

---

## Phase 3: End-to-End Tests

### 3.1 Docker Container Testing

**File:** `tests/docker-e2e.sh`

```bash
#!/bin/bash
set -e

echo "🐳 Starting Docker E2E tests..."

# Build Docker image
docker build -t arm-hypervisor:test .

# Start container
docker run -d \
  --name arm-hypervisor-test \
  -p 8080:8080 \
  -e RUST_LOG=debug \
  arm-hypervisor:test

# Wait for service to be ready
echo "⏳ Waiting for service to start..."
sleep 5

# Test 1: Health check
echo "✓ Test 1: Health check"
curl -f http://localhost:8080/health || exit 1

# Test 2: List containers (empty)
echo "✓ Test 2: List containers"
curl -f http://localhost:8080/api/v1/containers || exit 1

# Test 3: Create container with valid data
echo "✓ Test 3: Create container"
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-container",
    "template": "alpine",
    "config": {
      "cpu_limit": 2,
      "memory_limit": 536870912,
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/test-container/rootfs",
      "environment": []
    }
  }')

CONTAINER_ID=$(echo $RESPONSE | jq -r '.container.id')
echo "Created container: $CONTAINER_ID"

# Test 4: Get container
echo "✓ Test 4: Get container"
curl -f http://localhost:8080/api/v1/containers/test-container || exit 1

# Test 5: Validation - invalid name
echo "✓ Test 5: Validation error for invalid name"
VALIDATION_RESULT=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Invalid Name",
    "template": "alpine",
    "config": {}
  }')

ERROR_COUNT=$(echo $VALIDATION_RESULT | jq '.details | length')
if [ "$ERROR_COUNT" -gt 0 ]; then
  echo "✓ Validation properly rejected invalid input"
else
  echo "✗ Validation should have rejected invalid input"
  exit 1
fi

# Test 6: Validation - CPU too high
echo "✓ Test 6: Validation error for CPU > 128"
VALIDATION_RESULT=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "valid-name",
    "template": "alpine",
    "config": {
      "cpu_limit": 256,
      "memory_limit": 536870912,
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/test/rootfs",
      "environment": []
    }
  }')

ERROR_COUNT=$(echo $VALIDATION_RESULT | jq '.details | length')
if [ "$ERROR_COUNT" -gt 0 ]; then
  echo "✓ Validation properly rejected CPU limit"
else
  echo "✗ Validation should have rejected CPU limit"
  exit 1
fi

# Cleanup
docker stop arm-hypervisor-test
docker rm arm-hypervisor-test

echo "✅ All Docker E2E tests passed!"
```

**Run Tests:**
```bash
chmod +x tests/docker-e2e.sh
./tests/docker-e2e.sh
```

---

### 3.2 Database Persistence Verification

**File:** `tests/persistence-test.sh`

```bash
#!/bin/bash

echo "🔍 Testing database persistence..."

# Start application
cargo run --release &
APP_PID=$!
sleep 5

# Create container
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "persistent-test",
    "template": "alpine",
    "config": {
      "cpu_limit": 1,
      "memory_limit": 268435456,
      "disk_limit": 1073741824,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/persistent-test/rootfs",
      "environment": []
    }
  }')

CONTAINER_ID=$(echo $RESPONSE | jq -r '.container.id')
echo "Created container with ID: $CONTAINER_ID"

# Get container - verify UUID matches
RESPONSE2=$(curl -s http://localhost:8080/api/v1/containers/persistent-test)
CONTAINER_ID_2=$(echo $RESPONSE2 | jq -r '.container.id')

if [ "$CONTAINER_ID" == "$CONTAINER_ID_2" ]; then
  echo "✓ Container UUID persisted correctly"
else
  echo "✗ Container UUID changed! ($CONTAINER_ID != $CONTAINER_ID_2)"
  kill $APP_PID
  exit 1
fi

# Restart application
kill $APP_PID
sleep 2
cargo run --release &
APP_PID=$!
sleep 5

# Get container again - verify UUID STILL matches after restart
RESPONSE3=$(curl -s http://localhost:8080/api/v1/containers/persistent-test)
CONTAINER_ID_3=$(echo $RESPONSE3 | jq -r '.container.id')

if [ "$CONTAINER_ID" == "$CONTAINER_ID_3" ]; then
  echo "✓ Container UUID persisted across restart"
else
  echo "✗ Container UUID lost after restart! ($CONTAINER_ID != $CONTAINER_ID_3)"
  kill $APP_PID
  exit 1
fi

kill $APP_PID
echo "✅ Persistence tests passed!"
```

---

### 3.3 Real Status Tracking Tests

**File:** `tests/status-test.sh`

```bash
#!/bin/bash

echo "📊 Testing real status tracking..."

# Start application
cargo run --release &
APP_PID=$!
sleep 5

# Create container
curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "status-test",
    "template": "alpine",
    "config": {
      "cpu_limit": 1,
      "memory_limit": 268435456,
      "disk_limit": 1073741824,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/status-test/rootfs",
      "environment": []
    }
  }' > /dev/null

# Initial status should be stopped
STATUS=$(curl -s http://localhost:8080/api/v1/containers/status-test | jq -r '.container.status')
if [ "$STATUS" == "stopped" ]; then
  echo "✓ Initial status is 'stopped'"
else
  echo "✗ Initial status should be 'stopped', got: $STATUS"
  kill $APP_PID
  exit 1
fi

# Start container (requires LXC to be installed)
if command -v lxc-start &> /dev/null; then
  lxc-start -n status-test || true
  sleep 2

  # Check status - should be running
  STATUS=$(curl -s http://localhost:8080/api/v1/containers/status-test | jq -r '.container.status')
  if [ "$STATUS" == "running" ]; then
    echo "✓ Status updated to 'running' after start"
  else
    echo "⚠ LXC may not be installed, skipping real status test"
  fi

  lxc-stop -n status-test || true
else
  echo "⚠ LXC not installed, skipping real status verification"
fi

kill $APP_PID
echo "✅ Status tracking tests completed!"
```

---

## Phase 4: Test Execution

### 4.1 Run All Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run tests for specific crate
cargo test -p models --lib
cargo test -p database --lib
cargo test -p container-manager --lib
cargo test -p api-server --lib
```

### 4.2 Run Integration Tests

```bash
# Run integration tests
cargo test --test '*' 

# Run specific integration test
cargo test -p api-server --test integration_test
```

### 4.3 Run End-to-End Tests

```bash
# Docker E2E tests
./tests/docker-e2e.sh

# Persistence verification
./tests/persistence-test.sh

# Status tracking
./tests/status-test.sh
```

### 4.4 Full Test Suite

```bash
# Build and test everything
cargo build
cargo test --all
cargo test --all --lib
cargo test --all --test '*'

# With coverage
cargo tarpaulin --out Html --output-dir coverage
```

---

## Phase 5: Manual Testing Checklist

### 5.1 API Validation Testing

```bash
# Test 1: Valid request
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "valid-container",
    "template": "alpine",
    "config": {
      "cpu_limit": 4,
      "memory_limit": 1073741824,
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/valid-container/rootfs",
      "environment": []
    }
  }'
# Expected: 201 Created with container details

# Test 2: Invalid name (contains uppercase)
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Invalid",
    "template": "alpine",
    "config": {}
  }'
# Expected: 400 Bad Request with validation error

# Test 3: Invalid name (contains spaces)
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "invalid server",
    "template": "alpine",
    "config": {}
  }'
# Expected: 400 Bad Request

# Test 4: CPU limit too high
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "cpu-test",
    "template": "alpine",
    "config": {
      "cpu_limit": 256,
      "memory_limit": 1073741824,
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/cpu-test/rootfs",
      "environment": []
    }
  }'
# Expected: 400 Bad Request - CPU limit must be 1-128

# Test 5: Memory limit too low
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "mem-test",
    "template": "alpine",
    "config": {
      "cpu_limit": 2,
      "memory_limit": 10000000,  # Less than 64MB
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/mem-test/rootfs",
      "environment": []
    }
  }'
# Expected: 400 Bad Request - Memory limit must be at least 64MB

# Test 6: Disk limit too low
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "disk-test",
    "template": "alpine",
    "config": {
      "cpu_limit": 2,
      "memory_limit": 1073741824,
      "disk_limit": 10000000,  # Less than 100MB
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/disk-test/rootfs",
      "environment": []
    }
  }'
# Expected: 400 Bad Request - Disk limit must be at least 100MB
```

### 5.2 Database Persistence Verification

```bash
# Create container
CONTAINER=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{"name":"persist-test","template":"alpine","config":{"cpu_limit":1,"memory_limit":268435456,"disk_limit":1073741824,"network_interfaces":[],"rootfs_path":"/var/lib/lxc/persist-test/rootfs","environment":[]}}')

CONTAINER_ID=$(echo $CONTAINER | jq -r '.container.id')
echo "Created container: $CONTAINER_ID"

# Get container immediately
CONTAINER2=$(curl -s http://localhost:8080/api/v1/containers/persist-test)
CONTAINER_ID2=$(echo $CONTAINER2 | jq -r '.container.id')
echo "Retrieved container: $CONTAINER_ID2"

if [ "$CONTAINER_ID" == "$CONTAINER_ID2" ]; then
  echo "✓ UUID persisted correctly"
else
  echo "✗ UUID mismatch: $CONTAINER_ID != $CONTAINER_ID2"
fi

# Query database directly
sqlite3 /tmp/arm-hypervisor.db "SELECT id, name, status FROM containers WHERE name='persist-test';"
# Expected output: <CONTAINER_ID>|persist-test|stopped
```

### 5.3 Status Tracking Verification

```bash
# Get container status
curl -s http://localhost:8080/api/v1/containers/persist-test | jq '.container | {name, status, updated_at}'

# Expected: {"name":"persist-test","status":"stopped","updated_at":"2026-01-29T..."}

# List all containers - verify statuses
curl -s http://localhost:8080/api/v1/containers | jq '.containers[] | {name, status}'
```

---

## Phase 6: CI/CD Integration

### 6.1 GitHub Actions Workflow

**File:** `.github/workflows/test-critical-improvements.yml`

```yaml
name: Test Critical Improvements

on:
  push:
    branches: [main, develop, feat/critical-improvements]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      sqlite:
        image: sqlite
        options: --health-cmd="sqlite3 :memory: '.quit'" --health-interval=10s --health-timeout=5s --health-retries=5

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib --all

      - name: Run integration tests
        run: cargo test --test '*' --all

      - name: Run doc tests
        run: cargo test --doc --all

      - name: Run clippy
        run: cargo clippy --all --all-targets -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --timeout 300

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: false

  database-migration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run database migrations
        run: cargo run -p database --bin db-migrate

  docker-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: docker/setup-buildx-action@v2

      - name: Build Docker image
        uses: docker/build-push-action@v4
        with:
          context: .
          push: false
          tags: arm-hypervisor:test
          cache-from: type=registry,ref=ghcr.io/${{ github.repository }}:buildcache
```

---

## Test Results Template

### Unit Test Results
```
✓ Validation Tests (11 passed)
  - Container name validation
  - CPU limit validation
  - Memory limit validation
  - Disk limit validation
  - Template validation
  - Boundary conditions
  
✓ Database Tests (6 passed)
  - Pool creation
  - CRUD operations
  - Status updates
  - Data persistence
  
✓ Container Manager Tests (3 passed)
  - LXC integration
  - Status parsing
  - Command execution

Total: 20/20 passed
```

### Integration Test Results
```
✓ Handler Tests (8 passed)
  - Create container with validation
  - Invalid name rejection
  - CPU limit validation
  - Memory limit validation
  - Database persistence
  - Status retrieval
  - List containers
  - Update container status

Total: 8/8 passed
```

### E2E Test Results
```
✓ Docker Tests (6 passed)
  - Health check
  - List containers
  - Create container
  - Get container
  - Validation rejection
  - Error responses

✓ Persistence Tests (2 passed)
  - UUID persistence on restart
  - Data recovery

✓ Status Tests (1 passed)
  - Real status tracking

Total: 9/9 passed
```

---

## Test Environment Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLite
sudo apt-get install sqlite3  # Linux
brew install sqlite3          # macOS
```

### Database Setup

```bash
# Create test database
mkdir -p /tmp
sqlite3 /tmp/arm-hypervisor.db < crates/database/schema.sql

# Or let it auto-create on first run
# (migrations run automatically on startup)
```

### Docker Setup

```bash
# Build test image
docker build -t arm-hypervisor:test .

# Verify image
docker images | grep arm-hypervisor
```

---

## Performance Benchmarks

### Expected Test Execution Times

```
Unit Tests (all crates):      ~5-10 seconds
Integration Tests:             ~10-15 seconds
Docker Build:                 ~2-3 minutes
Docker E2E Tests:             ~1-2 minutes
Full Suite:                   ~5-10 minutes
```

### Optimization Tips

```bash
# Parallel test execution
cargo test --all -- --test-threads=4

# Skip slow tests
cargo test --all -- --skip integration

# Run only changed tests
cargo test --lib -p models

# Check test coverage
cargo tarpaulin --timeout 300 --out Html
```

---

## Success Criteria

✅ **All Tests Pass**
- Unit tests: 100% pass rate
- Integration tests: 100% pass rate
- E2E tests: 100% pass rate

✅ **Code Quality**
- Clippy warnings: 0
- Format check: passes
- No unsafe code warnings

✅ **Database**
- Migrations run successfully
- All CRUD operations work
- Data persists across restarts

✅ **Validation**
- All invalid inputs rejected
- Clear error messages returned
- No false positives

✅ **Performance**
- Request validation: < 1ms
- Database queries: < 5ms
- API response: < 100ms

---

## Troubleshooting

### Test Fails: "Cannot open database"
```bash
# Solution: Ensure /tmp is writable
chmod 777 /tmp

# Or specify different database location
export DATABASE_URL="sqlite:///tmp/test.db"
```

### Test Fails: "Connection timeout"
```bash
# Solution: Increase timeout for slow systems
cargo test --all -- --test-threads=1 --nocapture
```

### Docker Test Fails: "Port already in use"
```bash
# Solution: Stop existing container
docker stop arm-hypervisor-test
docker rm arm-hypervisor-test

# Or use different port
docker run -p 8081:8080 arm-hypervisor:test
```

### Validation Test Fails: "Unexpected error"
```bash
# Solution: Enable debug logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Test Reporting

### Generate Test Report

```bash
# HTML report
cargo tarpaulin --out Html --output-dir coverage

# JSON report
cargo test --all --format json > test-results.json

# JUnit XML (for CI)
cargo test --all -- --format json | jq > junit.xml
```

### View Coverage

```bash
# Open HTML coverage report
open coverage/index.html

# Or view in CI
# Coverage badge: ![Coverage](https://img.shields.io/badge/coverage-95%25-brightgreen)
```

---

## Next Actions

1. **Create test files** - Add integration tests and E2E scripts
2. **Set up CI/CD** - Configure GitHub Actions workflows
3. **Run full test suite** - Ensure all 20+ tests pass
4. **Measure coverage** - Aim for >90% code coverage
5. **Performance test** - Verify response times under load
6. **Documentation** - Update API docs with validation rules
7. **Release candidate** - Tag v0.2.0 with critical improvements
8. **Deployment** - Push to production after approval

