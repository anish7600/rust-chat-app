use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let (tx, mut rx) = mpsc::channel(10);

    // Spawn task to read user input
    tokio::spawn(async move {
        let mut input = String::new();
        let stdin = std::io::stdin();
        let mut stdin = BufReader::new(stdin);

        loop {
            input.clear();
            stdin.read_line(&mut input).await.unwrap();
            if input.trim().is_empty() {
                continue;
            }
            tx.send(input.clone()).await.unwrap();
        }
    });

    // Main loop for reading/writing
    loop {
        tokio::select! {
            // Read from server
            result = reader.read_line(&mut String::new()) => {
                if let Ok(n) = result {
                    if n == 0 {
                        break;
                    }
                    print!("{}", result.unwrap());
                }
            }
            // Send user input to server
            Some(msg) = rx.recv() => {
                writer.write_all(msg.as_bytes()).await?;
                writer.flush().await?;
            }
        }
    }

    Ok(())
}
