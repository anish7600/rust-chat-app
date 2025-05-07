use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Server running on 127.0.0.1:8080");

    let (tx, _rx) = broadcast::channel::<String>(10);

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New client connected: {}", addr);

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, tx, rx).await {
                error!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) -> Result<()> {
    let (reader, writer) = socket.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Task to forward broadcast messages to this client
    let writer_clone = Arc::clone(&writer);
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut writer = writer_clone.lock().await;
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Read messages from this client and broadcast them
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        info!("Received from client: {}", line.trim_end());
        tx.send(line.clone())?;
    }

    Ok(())
}
