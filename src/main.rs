use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}};
use rustra::ThreadPool;

fn main() {
    let port = 8080;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|e| panic!("Failed to bind to port {}: {}", port, e));

    let num_threads = num_cpus::get();
    let pool = ThreadPool::new(num_threads);

    println!("Server listening on http://127.0.0.1:{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    match buf_reader.read_line(&mut request_line) {
        Ok(_) => {
            // Each entry: (route path, HTTP status, HTML file)
            // Add new routes here; unmatched requests fall back to 404.
            let routes: &[(&str, &str, &str)] = &[
                ("/", "200 OK", "pages/index.html"),
            ];

            let (_, status, filename) = routes
                .iter()
                .find(|(path, _, _)| {
                    request_line.trim().starts_with(&format!("GET {} HTTP/1.1", path))
                })
                .copied()
                .unwrap_or(("", "404 Not Found", "pages/404.html"));

            match fs::read_to_string(filename) {
                Ok(contents) => {
                    let response = format!(
                        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
                        status,
                        contents.len(),
                        contents
                    );
                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        eprintln!("Failed to write response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file '{}': {}", filename, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read request line: {}", e);
        }
    }
}
