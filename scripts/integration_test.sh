#!/bin/bash

# Integration Test Script for moto-hses
# This script tests the communication between mock server and client

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MOCK_HOST="127.0.0.1"
MOCK_PORT="10040"
MOCK_PID=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="$PROJECT_ROOT/logs/integration-test.log"
MOCK_LOG_FILE="$PROJECT_ROOT/logs/mock-server.log"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Test result tracking
test_passed() {
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
    log_success "Test passed: $1"
}

test_failed() {
    ((TESTS_FAILED++))
    ((TESTS_TOTAL++))
    log_error "Test failed: $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    if [ ! -z "$MOCK_PID" ]; then
        log_info "Stopping mock server (PID: $MOCK_PID)"
        kill "$MOCK_PID" 2>/dev/null || true
        sleep 2
        # Force kill if still running
        kill -9 "$MOCK_PID" 2>/dev/null || true
        MOCK_PID=""
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Check if mock server is running
check_mock_server() {
    if netstat -tuln 2>/dev/null | grep -q ":$MOCK_PORT "; then
        return 0
    else
        return 1
    fi
}

# Start mock server
start_mock_server() {
    log_info "Starting mock server..."
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Ensure logs directory exists
    mkdir -p "$PROJECT_ROOT/logs"
    
    # Start mock server in background
    cargo run -p moto-hses-mock --example mock_basic_usage > "$MOCK_LOG_FILE" 2>&1 &
    MOCK_PID=$!
    
    # Wait for server to start
    log_info "Waiting for mock server to start..."
    for i in {1..30}; do
        if check_mock_server; then
            log_success "Mock server started successfully on $MOCK_HOST:$MOCK_PORT"
            return 0
        fi
        sleep 1
    done
    
    log_error "Mock server failed to start within 30 seconds"
    if [ -f "$MOCK_LOG_FILE" ]; then
        log_error "Mock server log:"
        cat "$MOCK_LOG_FILE"
    fi
    return 1
}

# Test basic client operations
test_basic_operations() {
    log_info "Testing basic client operations..."
    
    cd "$PROJECT_ROOT"
    
    # Run basic usage example and capture output
    local output
    if output=$(cargo run -p moto-hses-client --example basic_usage -- "$MOCK_HOST" "$MOCK_PORT" 2>&1); then
        # Check for expected success indicators
        if echo "$output" | grep -q "✓ Successfully connected to controller"; then
            test_passed "Client connection"
        else
            test_failed "Client connection - no success message"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Status read successfully"; then
            test_passed "Status reading"
        else
            test_failed "Status reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Position read successfully"; then
            test_passed "Position reading"
        else
            test_failed "Position reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm data read successfully"; then
            test_passed "Alarm data reading"
        else
            test_failed "Alarm data reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ D000 = 1"; then
            test_passed "Integer variable reading"
        else
            test_failed "Integer variable reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ R000 = 0"; then
            test_passed "Float variable reading"
        else
            test_failed "Float variable reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ B000 = 1"; then
            test_passed "Byte variable reading"
        else
            test_failed "Byte variable reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Robot running: true"; then
            test_passed "Convenience method - is_running"
        else
            test_failed "Convenience method - is_running"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Servo on: true"; then
            test_passed "Convenience method - is_servo_on"
        else
            test_failed "Convenience method - is_servo_on"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Has alarm: true"; then
            test_passed "Convenience method - has_alarm"
        else
            test_failed "Convenience method - has_alarm"
            return 1
        fi
        
        if echo "$output" | grep -q "Example completed successfully"; then
            test_passed "Basic operations completion"
        else
            test_failed "Basic operations completion"
            return 1
        fi
        
        log_success "Basic operations test completed successfully"
        return 0
    else
        test_failed "Basic operations execution"
        log_error "Basic operations output:"
        echo "$output"
        return 1
    fi
}

# Test alarm operations
test_alarm_operations() {
    log_info "Testing alarm operations..."
    
    cd "$PROJECT_ROOT"
    
    # Run alarm operations example and capture output
    local output
    if output=$(cargo run -p moto-hses-client --example alarm_operations -- "$MOCK_HOST" "$MOCK_PORT" 2>&1); then
        # Check for expected success indicators
        if echo "$output" | grep -q "✓ Complete alarm data read successfully"; then
            test_passed "Complete alarm data reading"
        else
            test_failed "Complete alarm data reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm code: 1001"; then
            test_passed "Alarm code reading"
        else
            test_failed "Alarm code reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm data: 1"; then
            test_passed "Alarm data reading"
        else
            test_failed "Alarm data reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm type: 1"; then
            test_passed "Alarm type reading"
        else
            test_failed "Alarm type reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm time: 2024/01/01 12:00"; then
            test_passed "Alarm time reading"
        else
            test_failed "Alarm time reading"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm name: Servo Error"; then
            test_passed "Alarm name reading"
        else
            test_failed "Alarm name reading"
            return 1
        fi
        
        # Check multiple alarm instances
        if echo "$output" | grep -q "✓ Alarm instance 1: Code=1001, Name=Servo Error"; then
            test_passed "Alarm instance 1"
        else
            test_failed "Alarm instance 1"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Alarm instance 2: Code=2001, Name=Emergency Stop"; then
            test_passed "Alarm instance 2"
        else
            test_failed "Alarm instance 2"
            return 1
        fi
        
        # Check alarm history reading (0x71 command)
        if echo "$output" | grep -q "✓ Major failure alarm 1: Code=1001, Name="; then
            test_passed "Alarm history - Major failure alarm 1"
        else
            test_failed "Alarm history - Major failure alarm 1"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Monitor alarm 1001: No alarm"; then
            test_passed "Alarm history - Monitor alarm 1001"
        else
            test_failed "Alarm history - Monitor alarm 1001"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Major failure alarm #1 code: 1001"; then
            test_passed "Alarm history - Code attribute"
        else
            test_failed "Alarm history - Code attribute"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Major failure alarm #1 time: 2024/01/01 12:00"; then
            test_passed "Alarm history - Time attribute"
        else
            test_failed "Alarm history - Time attribute"
            return 1
        fi
        
        if echo "$output" | grep -q "✓ Invalid instance correctly returned empty data"; then
            test_passed "Alarm history - Invalid instance handling"
        else
            test_failed "Alarm history - Invalid instance handling"
            return 1
        fi
        
        if echo "$output" | grep -q "Alarm operations example completed successfully"; then
            test_passed "Alarm operations completion"
        else
            test_failed "Alarm operations completion"
            return 1
        fi
        
        log_success "Alarm operations test completed successfully"
        return 0
    else
        test_failed "Alarm operations execution"
        log_error "Alarm operations output:"
        echo "$output"
        return 1
    fi
}

# Test connection management
test_connection_management() {
    log_info "Testing connection management..."
    
    cd "$PROJECT_ROOT"
    
    # Run connection management example and capture output
    local output
    if output=$(cargo run -p moto-hses-client --example connection_management -- "$MOCK_HOST" "$MOCK_PORT" 2>&1); then
        # Check for expected success indicators
        if echo "$output" | grep -q "✓ Successfully connected"; then
            test_passed "Connection management"
        else
            test_failed "Connection management"
            return 1
        fi
        
        log_success "Connection management test completed successfully"
        return 0
    else
        test_failed "Connection management execution"
        log_error "Connection management output:"
        echo "$output"
        return 1
    fi
}


# Test read status
test_read_status() {
    log_info "Testing read status..."
    
    cd "$PROJECT_ROOT"
    
    # Run read status example and capture output
    local output
    if output=$(cargo run -p moto-hses-client --example read_status -- "$MOCK_HOST" "$MOCK_PORT" 2>&1); then
        # Check for expected success indicators
        if echo "$output" | grep -q "Successfully connected to controller"; then
            test_passed "Read status connection"
        else
            test_failed "Read status connection"
            return 1
        fi
        
        if echo "$output" | grep -q "Complete status information retrieved"; then
            test_passed "Read status execution"
        else
            test_failed "Read status execution"
            return 1
        fi
        
        if echo "$output" | grep -q "Data1 retrieved"; then
            test_passed "Read status data1"
        else
            test_failed "Read status data1"
            return 1
        fi
        
        if echo "$output" | grep -q "Data2 retrieved"; then
            test_passed "Read status data2"
        else
            test_failed "Read status data2"
            return 1
        fi
        
        if echo "$output" | grep -q "Running: true"; then
            test_passed "Read status result"
        else
            test_failed "Read status result"
            return 1
        fi
        
        log_success "Read status test completed successfully"
        return 0
    else
        test_failed "Read status execution"
        log_error "Read status output:"
        echo "$output"
        return 1
    fi
}

# Test file operations (optional)
test_file_operations() {
    log_info "Testing file operations..."
    
    cd "$PROJECT_ROOT"
    
    # Run file operations example and capture output
    local output
    if output=$(cargo run -p moto-hses-client --example file_operations -- "$MOCK_HOST" "10041" 2>&1); then
        # Check for expected success indicators
        if echo "$output" | grep -q "✓"; then
            test_passed "File operations"
        else
            test_failed "File operations"
            return 1
        fi
        
        log_success "File operations test completed successfully"
        return 0
    else
        log_warning "File operations test failed or not available (this is optional)"
        return 0  # Don't fail the entire test suite for optional tests
    fi
}

# Main test execution
main() {
    log_info "Starting moto-hses integration tests..."
    log_info "Project root: $PROJECT_ROOT"
    log_info "Log file: $LOG_FILE"
    
    # Ensure logs directory exists and clear log file
    mkdir -p "$PROJECT_ROOT/logs"
    > "$LOG_FILE"
    
    # Build all crates first
    log_info "Building all crates..."
    cd "$PROJECT_ROOT"
    if ! cargo build --all-features --workspace; then
        log_error "Build failed"
        exit 1
    fi
    
    # Run unit tests
    log_info "Running unit tests..."
    if ! cargo test --all-features --workspace; then
        log_error "Unit tests failed"
        exit 1
    fi
    
    # Start mock server
    if ! start_mock_server; then
        log_error "Failed to start mock server"
        exit 1
    fi
    
    # Run integration tests
    log_info "Running integration tests..."
    
    test_basic_operations || log_error "Basic operations test failed"
    test_alarm_operations || log_error "Alarm operations test failed"
    test_connection_management || log_error "Connection management test failed"
    test_read_status || log_error "Read status test failed"
    test_file_operations || log_warning "File operations test failed (optional)"
    
    # Print test summary
    log_info "Test Summary:"
    log_info "  Total tests: $TESTS_TOTAL"
    log_info "  Passed: $TESTS_PASSED"
    log_info "  Failed: $TESTS_FAILED"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        log_success "All tests passed!"
        exit 0
    else
        log_error "Some tests failed!"
        exit 1
    fi
}

# Run main function
main "$@"
