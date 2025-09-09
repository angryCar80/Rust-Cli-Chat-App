use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                println!("Client says: {}", msg);
                stream.write_all(b"Message received!\n").unwrap();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

fn disconnect_client(stream: &mut TcpStream) -> io::Result<()> {
    stream.shutdown(Shutdown::Both)?;
    println!("Client Disconnected from the srever.");

    Ok(())
}

fn main() {
    println!("Write a message: ");
    // Server in a separate thread
    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        println!("Server listening on 7878...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    thread::spawn(move || handle_client(&mut stream));
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(500));

    // Client
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        println!("Client connected to server!");

        loop {
            let mut msg = String::new();
            // print!("> ");
            io::stdout().flush().unwrap();

            if io::stdin().read_line(&mut msg).is_err() {
                println!("Error reading input");
                continue;
            }

            if msg.trim() == "quit" {
                println!("Exiting chat...");
                if let Err(e) = disconnect_client(&mut stream) {
                    eprintln!("Error While disconnecting: {}", e);
                }
                break;
            }

            if let Err(e) = stream.write_all(msg.as_bytes()) {
                println!("Error sending message: {}", e);
                break;
            }
        }
    } else {
        println!("Client could not connect.");
    }

    thread::sleep(Duration::from_secs(1));
}
