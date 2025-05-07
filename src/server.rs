use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server running on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel(10);

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    // Handle incoming messages from client
                    result = reader.read_line(&mut line) => {
                        if result.is_err() || line.is_empty() {
                            break;
                        }
                        let msg = format!("[{}]: {}", addr, line.trim());
                        tx.send(msg.clone()).unwrap_or(0);
                        line.clear();
                    }
                    // Broadcast messages to client
                    result = rx.recv() => {
                        if let Ok(msg) = result {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                            writer.write_all(b"\n").await.unwrap();
                            writer.flush().await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
