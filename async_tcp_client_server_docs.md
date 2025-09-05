# Async TCP Client-Server

## Overview
An asynchronous TCP communication system with separate client and server components, built using Tokio for async runtime. The system supports multiple concurrent clients with message broadcasting capabilities.

## Features
- Asynchronous TCP server and client
- Multiple concurrent client connections
- Real-time message broadcasting between clients
- Graceful connection handling and cleanup
- Comprehensive error handling and logging
- Non-blocking I/O operations

## Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Architecture

### Server Component (`src/server.rs`)

#### Key Features
- Listens on `127.0.0.1:8080`
- Spawns async tasks for each client connection
- Uses broadcast channels for message distribution
- Structured logging with tracing

#### Core Implementation
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _rx) = broadcast::channel::<String>(10);
    
    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let rx = tx.subscribe();
        
        tokio::spawn(async move {
            handle_client(socket, tx, rx).await
        });
    }
}
```

#### Client Handler
- **Function**: `handle_client(socket, tx, rx)`
- **Purpose**: Manages individual client connections
- **Features**:
  - Splits socket into reader and writer
  - Spawns task for forwarding broadcast messages
  - Reads client messages and broadcasts them
  - Handles client disconnection gracefully

### Client Component (`src/client.rs`)

#### Key Features
- Connects to server at `127.0.0.1:8080`
- Concurrent handling of user input and server messages
- Real-time message display
- Graceful connection termination

#### Architecture
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, writer) = stream.into_split();
    let (tx, rx) = mpsc::channel::<String>(10);
    
    // Spawn input handling task
    // Spawn message sending task
    // Handle server responses
}
```

#### Concurrent Tasks
1. **Input Handler**: Reads from stdin asynchronously
2. **Message Sender**: Forwards messages to server
3. **Response Handler**: Displays server messages

## Usage

### Starting the Server
```bash
cargo run --bin server
```
Expected output:
```
2024-01-01T10:00:00.000Z INFO Server running on 127.0.0.1:8080
```

### Connecting Clients
```bash
cargo run --bin client
```
Expected output:
```
Connected to server
```

### Message Flow
1. Client types message and presses Enter
2. Message is sent to server
3. Server broadcasts message to all connected clients
4. All clients receive and display the message

## Message Broadcasting

### Broadcast Channel
- **Type**: `tokio::sync::broadcast`
- **Capacity**: 10 messages
- **Behavior**: All connected clients receive every message

### Message Format
- Messages are sent as raw strings with newline terminators
- No special formatting or protocol headers
- Real-time delivery to all subscribers

## Error Handling

### Server-Side Errors
- Connection acceptance failures logged and continue operation
- Client handling errors logged per connection
- Broadcast send errors handled gracefully

### Client-Side Errors
- Connection failures reported to user
- Stdin read errors break input loop
- Network write errors terminate connection

## Testing

### Test Coverage
The project includes comprehensive integration tests:

#### Basic Communication Test
```rust
#[tokio::test]
async fn test_client_send_receive() -> Result<()>
```
- Sets up mock server
- Tests message sending and receiving
- Verifies message content

#### Connection Handling Tests
1. **Empty Input Test**: Handles empty message lines
2. **Disconnect Test**: Handles server disconnection
3. **Invalid Address Test**: Tests connection error handling
4. **Multiple Messages Test**: Tests sequential message handling

### Running Tests
```bash
cargo test
```

## Network Configuration

### Default Settings
- **Host**: `127.0.0.1` (localhost)
- **Port**: `8080`
- **Protocol**: TCP
- **Concurrency**: Unlimited client connections

### Customization
To change network settings, modify the constants in source files:
```rust
// Server
let listener = TcpListener::bind("127.0.0.1:8080").await?;

// Client
let stream = TcpStream::connect("127.0.0.1:8080").await?;
```

## File Structure
```
src/
├── server.rs        # TCP server implementation
├── client.rs        # TCP client implementation
└── tests/           # Integration tests (inline)
```

## Performance Considerations

### Async Benefits
- Non-blocking I/O operations
- Efficient resource usage
- High concurrency support
- Low memory footprint per connection

### Scalability
- Server can handle hundreds of concurrent connections
- Message broadcasting scales with client count
- Memory usage grows linearly with connections

## Logging

### Server Logging
- Client connections: `INFO` level
- Error conditions: `ERROR` level
- Message processing: `INFO` level

### Log Format
```
2024-01-01T10:00:00.000Z INFO New client connected: 127.0.0.1:xxxxx
2024-01-01T10:00:01.000Z INFO Received from client: Hello World
```

## Security Considerations

### Current Limitations
- No authentication mechanism
- No message encryption
- No rate limiting
- No input validation

### Potential Improvements
1. Add TLS encryption
2. Implement client authentication
3. Add rate limiting per client
4. Input sanitization and validation
5. Connection timeouts

## Extensions and Improvements
1. **Protocol Enhancement**: Add message types and headers
2. **Persistence**: Store message history
3. **Rooms/Channels**: Support multiple chat rooms
4. **User Management**: Add usernames and user lists
5. **File Transfer**: Support file sharing between clients