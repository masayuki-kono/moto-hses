# Architecture Design

## Overview

This document describes the high-level architecture of the Rust HSES client library, focusing on design principles, component relationships, and architectural patterns rather than implementation details.

The library is inspired by the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) and incorporates design insights from the Python implementation [fs100](https://github.com/fih-mobile/fs100).

## Core Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety and preventing runtime errors
2. **Async-First**: Modern async/await patterns for non-blocking I/O operations
3. **Zero-Copy**: Efficient memory usage to minimize allocations and improve performance
4. **Comprehensive Error Handling**: Structured error handling with clear error propagation
5. **Modular Design**: Clear separation of concerns across different layers
6. **Performance Optimization**: Efficient batch operations and connection management
7. **Testability**: Built-in support for testing with mock implementations

## Architecture Overview

This repository provides a Rust HSES client library with a layered architecture. The library consists of three main components, while the Application Layer represents user code that consumes this library:

```
┌─────────────────────────────────────────────────────────────┐
│              Application Layer (User Code)                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │Robot Control│ │File Ops     │ │Status Monitor│          │
│  │Applications │ │Applications │ │Applications │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Uses
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              moto-hses-client (This Repository)            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  • High-level API • Connection Management               ││
│  │  • File Operations • Batch Operations                   ││
│  │  • Error Handling • Robot Control APIs                  ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Uses
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              moto-hses-proto (This Repository)             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  • Message Serialization • Type Definitions            ││
│  │  • Command Structures • Variable Operations             ││
│  │  • HSES Protocol Implementation                         ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Uses
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              moto-hses-mock (This Repository)              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  • Mock Server • Test Scenarios • Assertions           ││
│  │  • Development and Testing Support                     ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### moto-hses-proto (Protocol Layer)

**Responsibility**: Low-level protocol implementation and data serialization

**Key Responsibilities**:

- HSES protocol message serialization/deserialization
- Type-safe variable definitions and operations
- Command structure definitions
- Error handling for protocol-level issues
- Position and coordinate system abstractions

**Design Patterns**:

- **Type Safety**: Generic types ensure compile-time safety for variable operations
- **Command Pattern**: Unified interface for all HSES commands
- **Error Propagation**: Structured error handling with detailed error types

### moto-hses-client (Client Layer)

**Responsibility**: High-level API and connection management

**Key Responsibilities**:

- Async client implementation with connection pooling
- High-level API for robot operations and control
- File operations (upload, download, list, delete)
- Batch operation optimization
- Error handling and retry logic
- Configuration management

**Design Patterns**:

- **Facade Pattern**: Simplified interface hiding protocol complexity
- **Connection Pool**: Efficient resource management for high-performance applications
- **Batch Optimization**: Automatic grouping of operations to reduce network overhead

### moto-hses-mock (Testing Layer)

**Responsibility**: Mock server implementation for testing and development

**Key Responsibilities**:

- Mock HSES server for unit and integration testing
- Configurable test scenarios and responses
- Test assertion utilities
- Protocol validation and verification

**Design Patterns**:

- **Mock Object Pattern**: Simulates real server behavior for testing
- **Strategy Pattern**: Configurable handlers for different command types
- **Test Builder Pattern**: Fluent API for setting up test scenarios

## Key Design Patterns

### 1. Layered Architecture

The library follows a strict layered architecture where each layer has a specific responsibility and communicates only with adjacent layers:

- **Application Layer**: User code that consumes this library (outside repository scope)
- **Client Layer**: High-level API, connection management, and file operations
- **Protocol Layer**: Low-level protocol implementation and serialization
- **Testing Layer**: Mock implementations and test utilities

### 2. Type-Safe Operations

Leverages Rust's type system to ensure compile-time safety for all variable operations, preventing runtime type errors and improving code reliability.

### 3. Efficient Batch Operations

Implements intelligent optimization for multiple variable operations:

- **Consecutive Grouping**: Automatically groups consecutive variables of the same type
- **Plural Commands**: Uses HSES plural commands to reduce network round trips
- **Automatic Padding**: Handles protocol-specific padding requirements

### 4. Comprehensive Error Handling

Structured error handling with clear error propagation:

- **Protocol Errors**: Low-level protocol and serialization issues
- **Client Errors**: Connection, timeout, and high-level operation failures
- **Mock Errors**: Testing and validation errors

### 5. Async-First Design

Modern async/await patterns throughout the library:

- Non-blocking I/O operations
- Efficient resource utilization
- Support for concurrent operations

## Testing Strategy

The library provides comprehensive testing support through the mock layer:

### Unit Testing

- Mock server for isolated component testing
- Type-safe test utilities and assertions
- Configurable test scenarios

### Integration Testing

- End-to-end workflow testing
- Protocol validation and verification
- Performance and reliability testing

## Future Enhancements

1. **Plugin System**: Extensible command support for custom operations
2. **Performance Monitoring**: Built-in metrics and profiling capabilities
3. **Configuration Management**: YAML/JSON configuration file support
4. **Logging Integration**: Structured logging with tracing support
