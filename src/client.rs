use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::io;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let (tx, mut rx) = mpsc::channel::<String>(10);

    // Spawn task to read user input
    tokio::spawn({
        let tx = tx.clone();
        async move {
            let stdin = io::stdin();
            let mut stdin = BufReader::new(stdin);
            let mut input = String::new();

            loop {
                input.clear();
                match stdin.read_line(&mut input).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if input.trim().is_empty() {
                            continue;
                        }
                        if tx.send(input.clone()).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stdin: {}", e);
                        break;
                    }
                }
            }
        }
    });

    // Spawn task to send messages to the server
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Failed to write to server: {}", e);
                break;
            }
            if let Err(e) = writer.flush().await {
                eprintln!("Failed to flush to server: {}", e);
                break;
            }
        }
    });

    // Read responses from server
    loop {
        let mut buffer = String::new();
        let bytes_read = reader.read_line(&mut buffer).await?;

        if bytes_read == 0 {
            println!("Server closed the connection.");
            break;
        }

        print!("{}", buffer);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_client_send_receive() -> Result<()> {
        // Start a mock server
        let listener = TcpListener::bind("127.0.0.1:8082").await?;
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).await.unwrap();
            writer.write_all(b"Received: ").await.unwrap();
            writer.write_all(buffer.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        });

        // Connect client
        let mut stream = TcpStream::connect("127.0.0.1:8082").await?;
        stream.write_all(b"Test message\n").await?;
        stream.flush().await?;

        let mut buffer = String::new();
        timeout(Duration::from_secs(1), stream.read_to_string(&mut buffer)).await??;

        assert!(buffer.contains("Received: Test message"));
        server.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_client_empty_input() -> Result<()> {
        // Mock server
        let listener = TcpListener::bind("127.0.0.1:8083").await?;
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let (reader, _) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).await.unwrap();
            assert_eq!(buffer.trim(), ""); // Should receive empty line
        });

        // Connect client and send empty input
        let mut stream = TcpStream::connect("127.0.0.1:8083").await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        server.await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_client_server_disconnect() -> Result<()> {
        // Start a mock server that immediately closes
        let listener = TcpListener::bind("127.0.0.1:8084").await?;
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            drop(socket); // Simulate server disconnect
        });

        // Connect client
        let stream = TcpStream::connect("127.0.0.1:8084").await?;
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Read from server, expect disconnect
        let mut buffer = String::new();
        let result = timeout(Duration::from_secs(1), reader.read_line(&mut buffer)).await?;
        assert_eq!(result.unwrap(), 0); // Should read 0 bytes (EOF)

        server.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_client_invalid_address() -> Result<()> {
        // Try connecting to a non-existent server
        let result = TcpStream::connect("127.0.0.1:9999").await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_client_multiple_messages() -> Result<()> {
        // Start a mock server that echoes multiple messages
        let listener = TcpListener::bind("127.0.0.1:8085").await?;
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();

            // Read and echo two messages
            for _ in 0..2 {
                buffer.clear();
                reader.read_line(&mut buffer).await.unwrap();
                writer.write_all(b"Echo: ").await.unwrap();
                writer.write_all(buffer.as_bytes()).await.unwrap();
                writer.flush().await.unwrap();
            }
        });

        // Connect client
        let mut stream = TcpStream::connect("127.0.0.1:8085").await?;
        stream.write_all(b"First message\n").await?;
        stream.flush().await?;
        stream.write_all(b"Second message\n").await?;
        stream.flush().await?;

        let mut buffer = String::new();
        timeout(Duration::from_secs(1), stream.read_to_string(&mut buffer)).await??;

        assert!(buffer.contains("Echo: First message"));
        assert!(buffer.contains("Echo: Second message"));
        server.abort();
        Ok(())
    }
}
