# API Design

## Overview

This document describes the API design for the Rust HSES client library, focusing on usability, type safety, and performance.

## Core API Design

### Client Creation

```rust
use moto_hses_client::{HsesClient, ClientConfig};
use std::time::Duration;

// Simple client creation
let client = HsesClient::new("192.168.1.100:10040").await?;

// With custom configuration
let config = ClientConfig::default()
    .with_timeout(Duration::from_millis(500))
    .with_retry_count(3)
    .with_retry_delay(Duration::from_millis(100));

let client = HsesClient::with_config("192.168.1.100:10040", config).await?;
```

### Variable Operations

#### Reading Variables

```rust
use moto_hses_proto::VariableType;

// Type-safe variable reading
let value: i32 = client.read_variable(1, VariableType::Int32).await?;
let value: f32 = client.read_variable(2, VariableType::Float32).await?;
let value: String = client.read_variable(3, VariableType::String).await?;
let value: Position = client.read_variable(4, VariableType::Position).await?;

// With timeout
let value: i32 = client
    .read_variable(1, VariableType::Int32)
    .timeout(Duration::from_millis(300))
    .await?;

// Batch reading
let values = client
    .read_variables(&[
        (1, VariableType::Int32),
        (2, VariableType::Float32),
        (3, VariableType::String),
    ])
    .await?;
```

#### Writing Variables

```rust
// Type-safe variable writing
client.write_variable(1, 42i32).await?;
client.write_variable(2, 3.14f32).await?;
client.write_variable(3, "Hello Robot".to_string()).await?;

let position = Position::new(100.0, 200.0, 300.0, 0.0, 0.0, 0.0);
client.write_variable(4, position).await?;

// Batch writing
let variables = vec![
    (1, 42i32),
    (2, 3.14f32),
    (3, "Hello Robot".to_string()),
];
client.write_variables(&variables).await?;
```

### Job Operations

```rust
// Execute a job
client.execute_job(1).await?;

// Execute job with parameters
let params = JobParameters::new()
    .with_parameter("speed", 50.0)
    .with_parameter("acceleration", 100.0);
client.execute_job_with_params(1, params).await?;

// Get job status
let status = client.get_job_status(1).await?;
match status {
    JobStatus::Running => println!("Job is running"),
    JobStatus::Completed => println!("Job completed"),
    JobStatus::Error(e) => println!("Job failed: {}", e),
}
```

### Status Operations

```rust
// Get robot status
let status = client.get_status().await?;
println!("Robot status: {:?}", status);

// Get specific status information
let is_ready = client.is_ready().await?;
let is_error = client.is_error().await?;
let error_code = client.get_error_code().await?;
```

## Advanced API Features

### Connection Management

```rust
// Manual connection management
let mut client = HsesClient::new("192.168.1.100:10040").await?;

// Check connection status
if client.is_connected().await? {
    println!("Connected to robot");
}

// Reconnect if needed
if !client.is_connected().await? {
    client.reconnect().await?;
}

// Close connection
client.close().await?;
```

### Error Handling

```rust
use moto_hses_client::{HsesClient, ClientError};

let result: Result<i32, ClientError> = client.read_variable(1, VariableType::Int32).await;

match result {
    Ok(value) => println!("Value: {}", value),
    Err(ClientError::TimeoutError) => {
        println!("Request timed out");
        // Implement retry logic
    }
    Err(ClientError::ConnectionError(e)) => {
        println!("Connection error: {}", e);
        // Implement reconnection logic
    }
    Err(ClientError::ProtocolError(e)) => {
        println!("Protocol error: {}", e);
        // Handle protocol violations
    }
    Err(e) => println!("Other error: {}", e),
}
```

### Batch Operations

```rust
// Batch read with error handling
let results = client
    .read_variables_batch(&[
        (1, VariableType::Int32),
        (2, VariableType::Float32),
        (3, VariableType::String),
    ])
    .await?;

for (var_num, result) in results.iter().enumerate() {
    match result {
        Ok(value) => println!("Variable {}: {:?}", var_num + 1, value),
        Err(e) => println!("Variable {} error: {}", var_num + 1, e),
    }
}

// Transaction-like operations
let transaction = client.transaction();
transaction.write_variable(1, 42i32)?;
transaction.write_variable(2, 3.14f32)?;
transaction.execute_job(1)?;
transaction.commit().await?;
```

### Event-Driven API

```rust
use moto_hses_client::{HsesClient, EventHandler};

struct MyEventHandler;

impl EventHandler for MyEventHandler {
    fn on_variable_changed(&self, var_num: u16, value: VariableValue) {
        println!("Variable {} changed to {:?}", var_num, value);
    }

    fn on_job_completed(&self, job_num: u16, result: JobResult) {
        println!("Job {} completed with result: {:?}", job_num, result);
    }

    fn on_error(&self, error: ClientError) {
        println!("Error occurred: {}", error);
    }
}

let handler = Box::new(MyEventHandler);
client.set_event_handler(handler).await?;

// Start listening for events
client.start_event_listener().await?;
```

## Configuration API

### Client Configuration

```rust
use moto_hses_client::{ClientConfig, RetryPolicy, TimeoutPolicy};

let config = ClientConfig::default()
    .with_timeout(Duration::from_millis(300))
    .with_retry_policy(RetryPolicy::exponential_backoff(3, Duration::from_millis(100)))
    .with_timeout_policy(TimeoutPolicy::adaptive(Duration::from_millis(100), Duration::from_secs(5)))
    .with_buffer_size(8192)
    .with_connection_pool_size(5);

let client = HsesClient::with_config("192.168.1.100:10040", config).await?;
```

### Environment Configuration

```rust
// Load configuration from environment variables
let config = ClientConfig::from_env()
    .with_default_host("192.168.1.100")
    .with_default_port(10040)
    .with_default_timeout(Duration::from_millis(300));

let client = HsesClient::with_config_from_env(config).await?;
```

## Type Definitions

### Variable Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Int32,
    Float32,
    String,
    Position,
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    Int32(i32),
    Float32(f32),
    String(String),
    Position(Position),
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
}
```

### Job Types

```rust
#[derive(Debug, Clone)]
pub struct JobParameters {
    pub parameters: HashMap<String, VariableValue>,
}

#[derive(Debug, Clone)]
pub enum JobStatus {
    NotStarted,
    Running,
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct JobResult {
    pub status: JobStatus,
    pub execution_time: Duration,
    pub error_message: Option<String>,
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] moto_hses_proto::ProtocolError),

    #[error("Invalid variable: {0}")]
    InvalidVariable(String),

    #[error("Invalid job: {0}")]
    InvalidJob(String),

    #[error("System error: {0}")]
    SystemError(String),
}
```

## Best Practices

### Error Handling

1. Always handle errors explicitly
2. Use appropriate error types for different scenarios
3. Implement retry logic for transient errors
4. Log errors with sufficient context

### Performance

1. Use batch operations for multiple variables
2. Reuse client instances when possible
3. Set appropriate timeouts
4. Use connection pooling for high-frequency operations

### Type Safety

1. Use strongly-typed variable operations
2. Leverage Rust's type system for compile-time safety
3. Use enums for variable types and status values
4. Implement proper validation for all inputs
