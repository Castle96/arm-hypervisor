# Testing Phase - Complete Guide

**Project:** ARM Hypervisor Platform  
**Date:** January 29, 2026  
**Scope:** Critical Improvements Testing  

---

## Executive Summary

The ARM Hypervisor project has successfully implemented three critical improvements:

1. ✅ **Database/Persistence Layer** - SQLite backend for persistent storage
2. ✅ **Real Container Status Tracking** - Queries actual LXC state
3. ✅ **Request Validation** - Comprehensive input validation

This document describes the complete testing strategy to validate these improvements before production deployment.

---

## Testing Phases Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Unit Tests (70% - Foundation)                      │
│ ✓ Validation module tests (11 tests)                        │
│ ✓ Database module tests (6+ tests)                          │
│ ✓ Container manager tests (3 tests)                         │
├─────────────────────────────────────────────────────────────┤
│ Phase 2: Integration Tests (15% - System)                   │
│ ✓ Handler integration tests                                 │
│ ✓ Database operations end-to-end                            │
│ ✓ Validation in context                                     │
├─────────────────────────────────────────────────────────────┤
│ Phase 3: E2E Tests (10% - Real World)                       │
│ ✓ Docker container tests                                    │
│ ✓ Persistence verification                                  │
│ ✓ Status tracking validation                                │
├─────────────────────────────────────────────────────────────┤
│ Phase 4: Manual Testing (5% - Edge Cases)                   │
│ ✓ API validation scenarios                                  │
│ ✓ Database queries                                          │
│ ✓ Performance benchmarks                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start Testing

### For Impatient Users (5 minutes)

```bash
# Run automated test suite
./scripts/run-tests.sh                    # Linux/macOS
.\scripts\run-tests.bat                   # Windows

# This runs:
# - All unit tests (20+)
# - Build verification
# - Code quality checks
# - Database migration test
# - Integration tests
```

### Expected Output
```
✓ All unit tests passed
✓ Project builds successfully
✓ Code quality checks passed
✓ Database migrations work correctly
✓ Integration tests completed

Testing phase: 5-10 minutes
All tests: 100% pass rate
```

---

## Detailed Testing Guide

### Phase 1: Unit Tests (10 minutes)

#### 1.1 Validation Tests
```bash
cargo test -p models --lib validation -- --nocapture
```

**What's tested:**
- Container name format validation
- CPU limit bounds (1-128)
- Memory limit constraints (64MB-1TB)
- Disk limit constraints (100MB-10TB)
- Template format validation
- Edge cases and boundaries

**Expected results:** 11/11 pass

#### 1.2 Database Tests
```bash
cargo test -p database --lib -- --nocapture
```

**What's tested:**
- Connection pool creation
- Container CRUD operations
- Status updates
- Data persistence
- Get by ID/name
- Existence checks
- Idempotent operations

**Expected results:** 6+/6+ pass

#### 1.3 Container Manager Tests
```bash
cargo test -p container-manager --lib -- --nocapture
```

**What's tested:**
- Container name validation
- Container state parsing
- LXC command execution
- Error handling

**Expected results:** 3+/3+ pass

#### 1.4 All Unit Tests
```bash
cargo test --lib --all
```

**Expected results:** 20+/20+ pass in <30 seconds

---

### Phase 2: Build Verification (3 minutes)

#### 2.1 Debug Build
```bash
cargo build
```

**Checks:**
- ✓ No compilation errors
- ✓ No compiler warnings (or documented)
- ✓ Builds in reasonable time

#### 2.2 Release Build
```bash
cargo build --release
```

**Checks:**
- ✓ Optimization flags applied
- ✓ Binary created
- ✓ Binary size < 100MB

#### 2.3 Incremental Build
```bash
cargo build  # Run twice
```

**Checks:**
- ✓ Incremental compilation works
- ✓ Second build < 5 seconds

---

### Phase 3: Code Quality (2 minutes)

#### 3.1 Formatting
```bash
cargo fmt -- --check
```

**Result:** No issues (all files follow Rust style)

#### 3.2 Linting
```bash
cargo clippy --all -- -D warnings
```

**Result:** No critical warnings

#### 3.3 Documentation
```bash
cargo doc --open
```

**Checks:**
- ✓ All public items documented
- ✓ Examples included
- ✓ Links working

---

### Phase 4: Database Testing (2 minutes)

#### 4.1 Migration Test
```bash
cargo run -p database --bin db-migrate
```

**Verification:**
- ✓ Migrations complete
- ✓ Database file created
- ✓ Schema validated

```bash
sqlite3 /tmp/arm-hypervisor.db ".tables"
# Expected: containers
```

#### 4.2 Schema Validation
```bash
sqlite3 /tmp/arm-hypervisor.db ".schema containers"
```

**Checks:**
- ✓ All columns present
- ✓ Types correct
- ✓ Constraints in place
- ✓ Indexes created

---

### Phase 5: Manual API Testing (10 minutes)

Start the server in one terminal:
```bash
cargo run --release
```

In another terminal, test the endpoints:

#### 5.1 Health Check
```bash
curl http://localhost:8080/health
```

**Expected:** 200 OK with status information

#### 5.2 List Containers (Empty)
```bash
curl http://localhost:8080/api/v1/containers
```

**Expected:** 200 OK with empty array

#### 5.3 Create Valid Container
```bash
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
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
  }'
```

**Expected:** 201 Created with container details including UUID

#### 5.4 Validation Test - Invalid Name
```bash
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Invalid Name",
    "template": "alpine",
    "config": {}
  }'
```

**Expected:** 400 Bad Request with validation error details

#### 5.5 Validation Test - CPU Too High
```bash
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test",
    "template": "alpine",
    "config": {
      "cpu_limit": 256,
      "memory_limit": 536870912,
      "disk_limit": 10737418240,
      "network_interfaces": [],
      "rootfs_path": "/var/lib/lxc/test/rootfs",
      "environment": []
    }
  }'
```

**Expected:** 400 Bad Request - "CPU limit must be between 1 and 128"

#### 5.6 Get Container
```bash
curl http://localhost:8080/api/v1/containers/web-server-1
```

**Expected:** 200 OK with container details, UUID matches creation response

#### 5.7 Verify Persistence
```bash
# Query database directly
sqlite3 /tmp/arm-hypervisor.db "SELECT id, name, status FROM containers;"
```

**Expected:** Container UUID and data persisted

---

## Validation Test Matrix

### Container Name Validation

| Input | Expected | Result |
|-------|----------|--------|
| `web-server-1` | ✅ Pass | Valid |
| `api.prod` | ✅ Pass | Valid |
| `test` | ✅ Pass | Valid |
| `Invalid Name` | ❌ Fail | Spaces not allowed |
| `Test_Server` | ❌ Fail | Underscore not allowed |
| `UPPERCASE` | ❌ Fail | Must be lowercase |
| Empty string | ❌ Fail | Cannot be empty |

### Resource Limits Validation

| Resource | Min | Max | Test Value | Result |
|----------|-----|-----|------------|--------|
| CPU | 1 | 128 | 0 | ❌ Fail |
| CPU | 1 | 128 | 256 | ❌ Fail |
| Memory | 64MB | 1TB | 32MB | ❌ Fail |
| Memory | 64MB | 1TB | 512MB | ✅ Pass |
| Disk | 100MB | 10TB | 50MB | ❌ Fail |
| Disk | 100MB | 10TB | 5GB | ✅ Pass |

---

## Performance Expectations

### Response Times
- **Validation:** < 1ms per field
- **Database query:** < 5ms per operation
- **API response:** < 100ms total
- **Container creation:** < 500ms

### Database Performance
- **Insert 1 record:** < 1ms
- **Query by name:** < 1ms (indexed)
- **List 100 containers:** < 10ms
- **Update status:** < 1ms

### Concurrent Load
- **10 simultaneous requests:** All succeed
- **100 simultaneous requests:** All succeed
- **Connection pool:** Handles 10+ concurrent

---

## Test Results Checklist

### Unit Tests
- [ ] All 20+ tests pass
- [ ] < 30 seconds total time
- [ ] No memory leaks
- [ ] No timeout issues

### Build Tests
- [ ] Debug build succeeds
- [ ] Release build succeeds
- [ ] Incremental build works
- [ ] Binary size acceptable

### Code Quality
- [ ] Clippy passes
- [ ] Formatting correct
- [ ] Documentation complete
- [ ] No unsafe code

### Manual Tests
- [ ] API health check works
- [ ] Valid creation succeeds
- [ ] Invalid inputs rejected
- [ ] Error messages clear
- [ ] Database persistence works
- [ ] UUID stays consistent
- [ ] Status tracking accurate

### Performance
- [ ] Validation < 1ms
- [ ] Queries < 5ms
- [ ] API response < 100ms
- [ ] Concurrent requests ok
- [ ] No memory leaks

---

## Common Issues & Solutions

### Issue: "Cannot open database"
```bash
# Solution: Ensure /tmp is writable
chmod 777 /tmp

# Or use different location
export DATABASE_URL="sqlite:////var/tmp/arm-hypervisor.db"
```

### Issue: "Port 8080 already in use"
```bash
# Solution 1: Kill existing process
lsof -i :8080 | grep -i listen | awk '{print $2}' | xargs kill -9

# Solution 2: Use different port
export PORT=8081
cargo run --release
```

### Issue: "Test timeout"
```bash
# Solution: Run with single thread and increase timeout
cargo test --lib -- --test-threads=1 --timeout 60
```

### Issue: "Validation always passes"
```bash
# Solution: Check error handling in handlers
# Make sure validation_errors vec is checked before creating container
```

---

## Documentation Links

1. **[CRITICAL-IMPROVEMENTS-IMPLEMENTATION.md](CRITICAL-IMPROVEMENTS-IMPLEMENTATION.md)**
   - Detailed implementation documentation
   - Architecture changes
   - Integration points

2. **[TESTING-CRITICAL-IMPROVEMENTS.md](TESTING-CRITICAL-IMPROVEMENTS.md)**
   - Complete testing strategy
   - Test code examples
   - CI/CD integration

3. **[TESTING-READINESS-CHECKLIST.md](TESTING-READINESS-CHECKLIST.md)**
   - Phase-by-phase checklist
   - Manual test scenarios
   - Sign-off template

---

## Test Execution Timeline

| Phase | Tasks | Time | Total |
|-------|-------|------|-------|
| 1 | Unit Tests | 10 min | 10 min |
| 2 | Build Verification | 3 min | 13 min |
| 3 | Code Quality | 2 min | 15 min |
| 4 | Database Testing | 2 min | 17 min |
| 5 | Manual Testing | 10 min | 27 min |
| 6 | Documentation | 3 min | 30 min |

**Total Testing Time: ~30 minutes**

---

## Success Criteria

✅ **All tests pass** (100% pass rate)
✅ **No critical warnings** (code quality acceptable)
✅ **Performance meets targets** (response times acceptable)
✅ **Data persists** (database working)
✅ **Validation works** (invalid inputs rejected)
✅ **API functional** (endpoints responding)
✅ **Documentation complete** (guides available)

---

## Next Steps

1. **Run Tests**
   ```bash
   ./scripts/run-tests.sh  # or .bat on Windows
   ```

2. **Review Results**
   - Check test output
   - Verify all pass
   - Note any warnings

3. **Manual Testing**
   - Test API endpoints
   - Verify validation
   - Check persistence

4. **Create Pull Request**
   - Link to test results
   - Reference critical improvements
   - Request review

5. **Deploy**
   - Merge when approved
   - Deploy to staging
   - Monitor logs

---

## Questions?

Refer to the detailed documentation files or create an issue with:
- Steps to reproduce
- Error messages
- Environment details
- Expected vs actual behavior

---

**Status:** 🟢 **READY FOR TESTING**

All files prepared, documentation complete, ready to execute test phases.

