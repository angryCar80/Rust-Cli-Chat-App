use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server!");

    let mut input = String::new();

    loop {
        input.clear();
        io::stdin().read_line(&mut input)?;

        // send to server
        stream.write_all(input.as_bytes()).await?;

        // receive from server
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await?;

        if n == 0 {
            println!("Server closed connection.");
            break;
        }

        println!("Server says: {}", String::from_utf8_lossy(&buf[..n]));
    }

    Ok(())
}

