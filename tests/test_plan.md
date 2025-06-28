# Petra Test Plan

## Unit Tests (in each module)
- [x] Value type conversions
- [x] Signal bus basic operations
- [x] Block logic execution
- [x] Config parsing and validation
- [ ] Validation rules
- [ ] Security authentication
- [ ] Alarm conditions

## Integration Tests

### 1. End-to-End Workflow (`tests/integration/e2e_test.rs`)
- [ ] Load config → Start engine → Process signals → Verify outputs
- [ ] MQTT publish/subscribe with real broker
- [ ] S7 simulation tests
- [ ] Storage write and query

### 2. Performance Tests (`tests/integration/performance_test.rs`)
- [ ] 10k signals at 50ms scan time
- [ ]
