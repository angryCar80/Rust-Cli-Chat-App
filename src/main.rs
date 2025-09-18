// the main question is  how to brodcast ?

use colored::Colorize;
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::Arc,
    sync::Mutex,
    thread,
    time::Duration,
};

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                println!("{} {}", "[CLIENT]".blue(), msg.blue());
                let mut locked = clients.lock().unwrap();
                locked.retain(|s| s.peer_addr().is_ok());

                for client in locked.iter_mut() {
                    if client.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                        let _ = client.write_all(format!("{}\n", msg).as_bytes());
                    }
                }
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
        println!("{}", "Server listening on 7878...".green());
        let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients_clone = Arc::clone(&clients);

                    clients.lock().unwrap().push(stream.try_clone().unwrap());
                    thread::spawn(move || handle_client(stream, clients_clone));
                }
                Err(e) => println!("Connectio faild: {}", e),
            }
        }
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(500));

    // Client
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        println!("Client connected to server!");
        let mut stream_clone = stream.try_clone().expect("Faild to clone stream");
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

                        println!("[SERVER]: {}", msg);
                        println!(" {} {}", "[SERVER]".green(), msg.green());
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
