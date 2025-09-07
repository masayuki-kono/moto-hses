#!/bin/bash

# Integration Test Script for moto-hses
# This script tests the communication between mock server and client
#
# Usage:
#   ./integration_test.sh                    # Run tests in normal mode
#   DEBUG_MODE=true ./integration_test.sh    # Run tests with debug output
#
# Features:
#   - Configuration-based testing using test_config.yaml
#   - Structured output validation
#   - Debug mode for detailed error information
#   - Automatic cleanup of temporary files
#   - Comprehensive test result reporting

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
LOG_FILE="$PROJECT_ROOT/logs/integration_test_summary.log"
MOCK_LOG_FILE="$PROJECT_ROOT/logs/mock_server.log"
TEST_CONFIG_FILE="$SCRIPT_DIR/integration_test.toml"

# Debug mode
DEBUG_MODE=${DEBUG_MODE:-false}

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

# Debug output function
debug_output() {
    if [ "$DEBUG_MODE" = "true" ]; then
        echo -e "${YELLOW}[DEBUG]${NC} $1" | tee -a "$LOG_FILE"
    fi
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

# Enhanced test failure with details
test_failed_with_details() {
    local test_name="$1"
    local expected="$2"
    local actual="$3"
    
    test_failed "$test_name"
    if [ "$DEBUG_MODE" = "true" ]; then
        log_error "Expected: $expected"
        log_error "Actual: $actual"
    fi
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

# TOML parsing functions (simple implementation)
parse_toml() {
    local toml_file="$1"
    local test_name="$2"
    
    # Extract expected outputs for the given test
    awk -v test_name="$test_name" '
    BEGIN { in_test = 0; in_outputs = 0 }
    /^\[tests\./ { 
        if ($0 ~ "\\[tests\\." test_name "\\]") {
            in_test = 1
            next
        } else if (in_test) {
            in_test = 0
            in_outputs = 0
        }
    }
    in_test && /^\[\[tests\./ {
        if ($0 ~ "\\[\\[tests\\." test_name "\\.expected_outputs\\]\\]") {
            in_outputs = 1
            next
        } else if (in_outputs) {
            in_outputs = 0
        }
    }
    in_outputs && /^type =/ { 
        gsub(/^type = /, "", $0)
        gsub(/^"/, "", $0)
        gsub(/"$/, "", $0)
        print "type: " $0
    }
    in_outputs && /^pattern =/ { 
        gsub(/^pattern = /, "", $0)
        gsub(/^"/, "", $0)
        gsub(/"$/, "", $0)
        print "pattern: " $0
    }
    in_outputs && /^description =/ { 
        gsub(/^description = /, "", $0)
        gsub(/^"/, "", $0)
        gsub(/"$/, "", $0)
        print "description: " $0
    }
    ' "$toml_file" | sed 's/^"//' | sed 's/"$//'
}

# Get list of all test names from TOML config
get_all_test_names() {
    grep -E '^\[tests\.' "$TEST_CONFIG_FILE" | sed 's/^\[tests\.//' | sed 's/\]$//'
}

# Get test configuration
get_test_config() {
    local test_name="$1"
    local config_key="$2"
    
    case "$config_key" in
        "command")
            awk -v test_name="$test_name" '
            /^\[tests\./ { 
                if ($0 ~ "\\[tests\\." test_name "\\]") {
                    in_test = 1
                    next
                } else {
                    in_test = 0
                }
            }
            in_test && /^command =/ { 
                gsub(/^command = /, "", $0)
                gsub(/^"/, "", $0)
                gsub(/"$/, "", $0)
                print $0
            }
            ' "$TEST_CONFIG_FILE" | sed 's/^"//' | sed 's/"$//'
            ;;
        "port")
            local port_value=$(awk -v test_name="$test_name" '
            /^\[tests\./ { 
                if ($0 ~ "\\[tests\\." test_name "\\]") {
                    in_test = 1
                    next
                } else {
                    in_test = 0
                }
            }
            in_test && /^port =/ { 
                gsub(/^port = /, "", $0)
                gsub(/^"/, "", $0)
                gsub(/"$/, "", $0)
                print $0
            }
            ' "$TEST_CONFIG_FILE" | sed 's/^"//' | sed 's/"$//')
            if [ -z "$port_value" ]; then
                log_error "Port not specified for test: $test_name"
                exit 1
            fi
            echo "$port_value"
            ;;
        "optional")
            awk -v test_name="$test_name" '
            /^\[tests\./ { 
                if ($0 ~ "\\[tests\\." test_name "\\]") {
                    in_test = 1
                    next
                } else {
                    in_test = 0
                }
            }
            in_test && /^optional =/ { 
                if ($0 ~ "true") print "true"
                else print "false"
            }
            ' "$TEST_CONFIG_FILE" | head -1 || echo "false"
            ;;
    esac
}

# Generic test output validation
validate_test_output() {
    local test_name="$1"
    local output_file="$2"
    local temp_dir="$PROJECT_ROOT/logs/integration_test_detailed_outputs"
    
    mkdir -p "$temp_dir"
    local expected_file="$temp_dir/expected_${test_name}.log"
    
    # Extract expected outputs from TOML config
    parse_toml "$TEST_CONFIG_FILE" "$test_name" > "$expected_file"
    
    local passed=0
    local failed=0
    local current_type=""
    local current_pattern=""
    local current_description=""
    
    # Debug information only in debug mode
    if [ "$DEBUG_MODE" = "true" ]; then
        debug_output "Validating test: $test_name"
        debug_output "Output file: $output_file"
        debug_output "Expected file: $expected_file"
        
        # Show expected file contents
        if [ -f "$expected_file" ]; then
            debug_output "Expected file contents:"
            cat "$expected_file" | while read line; do
                debug_output "  $line"
            done
        fi
    fi
    
    while IFS= read -r line; do
        if [[ "$line" =~ ^type: ]]; then
            current_type=$(echo "$line" | sed 's/type: //')
        elif [[ "$line" =~ ^pattern: ]]; then
            current_pattern=$(echo "$line" | sed 's/pattern: //')
        elif [[ "$line" =~ ^description: ]]; then
            current_description=$(echo "$line" | sed 's/description: //')
            
            # Now we have all three pieces, validate
            if [ -n "$current_type" ] && [ -n "$current_pattern" ] && [ -n "$current_description" ]; then
                if grep -q "$current_pattern" "$output_file"; then
                    test_passed "$test_name: $current_description"
                    ((passed++))
                else
                    test_failed_with_details "$test_name: $current_description" "$current_pattern" "$(head -5 "$output_file")"
                    ((failed++))
                fi
                
                # Reset for next iteration
                current_type=""
                current_pattern=""
                current_description=""
            fi
        fi
    done < "$expected_file"
    
    # Only show validation summary in debug mode
    if [ "$DEBUG_MODE" = "true" ]; then
        debug_output "$test_name validation: $passed passed, $failed failed"
    fi
    
    # Cleanup expected file only (keep output files for debugging)
    rm -f "$expected_file"
    
    return $failed
}

# Run test with configuration
run_configured_test() {
    local test_name="$1"
    local command=$(get_test_config "$test_name" "command")
    local port=$(get_test_config "$test_name" "port")
    local optional=$(get_test_config "$test_name" "optional")
    
    log_info "Testing $test_name..."
    if [ "$DEBUG_MODE" = "true" ]; then
        debug_output "Command: $command, Port: $port, Optional: $optional"
    fi
    
    cd "$PROJECT_ROOT"
    
    # Create temporary output directory and file
    local temp_dir="$PROJECT_ROOT/logs/integration_test_detailed_outputs"
    mkdir -p "$temp_dir"
    local temp_output="$temp_dir/${test_name}.log"
    
    # Run the test command
    if cargo run -p moto-hses-client --example $command -- "$MOCK_HOST" "$port" > "$temp_output" 2>&1; then
        # Validate output
        if validate_test_output "$test_name" "$temp_output"; then
            log_success "$test_name test completed successfully"
            # Keep output file for debugging in logs directory
            return 0
        else
            log_error "$test_name test failed validation"
            log_error "Detailed output available in: $temp_output"
            # Keep output file for debugging in logs directory
            return 1
        fi
    else
        if [ "$optional" = "true" ]; then
            log_warning "$test_name test failed (optional test)"
            rm -f "$temp_output"
            return 0
        else
            test_failed "$test_name execution"
            log_error "Detailed output available in: $temp_output"
            # Keep output file for debugging in logs directory
            return 1
        fi
    fi
}

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

# Run all configured tests
run_all_tests() {
    local test_names
    test_names=$(get_all_test_names)
    
    if [ -z "$test_names" ]; then
        log_error "No tests found in configuration file"
        return 1
    fi
    
    log_info "Found tests: $(echo $test_names | tr '\n' ' ')"
    
    for test_name in $test_names; do
        run_configured_test "$test_name"
    done
}

# Main test execution
main() {
    log_info "Starting moto-hses integration tests..."
    log_info "Project root: $PROJECT_ROOT"
    log_info "Log file: $LOG_FILE"
    log_info "Test config: $TEST_CONFIG_FILE"
    log_info "Debug mode: $DEBUG_MODE"
    
    # Check if test configuration file exists
    if [ ! -f "$TEST_CONFIG_FILE" ]; then
        log_error "Test configuration file not found: $TEST_CONFIG_FILE"
        log_error "Please ensure the integration_test.toml file exists in the scripts directory"
        exit 1
    fi
    
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
    
    run_all_tests
    
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
