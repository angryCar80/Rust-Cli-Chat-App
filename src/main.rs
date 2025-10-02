use colored::Colorize;
use std::{
    io::{self, Read, Write, stdout},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct Client {
    username: String,
    stream: TcpStream,
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>, username: String) {
    let mut buffer = [0; 1024]; // the user input buffer
    loop {
        match stream.read(&mut buffer) {
            // Checking user status
            Ok(0) => {
                println!("{} disconnected", username.red());
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                println!("{} {}", format!("[{}]", username).blue(), msg.blue());

                let mut locked = clients.lock().unwrap();
                locked.retain(|c| c.stream.peer_addr().is_ok());

                for client in locked.iter_mut() {
                    if client.stream.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                        let _ = client
                            .stream
                            .write_all(format!("{}: {}\n", username, msg).as_bytes());
                    }
                }
            }
            // Handling Error
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

fn disconnect_client(stream: &mut TcpStream) -> io::Result<()> {
    stream.shutdown(Shutdown::Both)?;
    println!("Client Disconnected from the server.");
    Ok(())
}

fn main() {
    print!("Enter your name: ");
    stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let _username = username.trim().to_string();

    println!("Write a message: ");

    // Server in a separate thread
    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        println!("{}", "Server listening on 7878...".green());
        let clients = Arc::new(Mutex::new(Vec::<Client>::new()));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients_clone = Arc::clone(&clients);

                    // For now, assign "Anonymous" since server doesn't know username
                    let username = "Anonymous".to_string();

                    clients.lock().unwrap().push(Client {
                        username: username.clone(),
                        stream: stream.try_clone().unwrap(),
                    });

                    thread::spawn(move || handle_client(stream, clients_clone, username));
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(500));

    // Client
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        println!("Client connected to server!");
        let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
        thread::spawn(move || {
            let mut buffer = [0; 512];
            loop {
                match stream_clone.read(&mut buffer) {
                    Ok(0) => {
                        println!("Server Closed.");
                        break;
                    }
                    Ok(n) => {
                        let msg = String::from_utf8_lossy(&buffer[..n]);
                        println!("{} {}", "[SERVER]".green(), msg.green());
                        print!("> ");
                        io::stdout().flush().unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error from the server: {}", e);
                        break;
                    }
                }
            }
        });

        loop {
            let mut msg = String::new();
            print!("> ");
            io::stdout().flush().unwrap();

            if io::stdin().read_line(&mut msg).is_err() {
                println!("{}", "Error reading input".red());
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
