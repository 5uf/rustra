mod lib;

use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}};
use lib::ThreadPool;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
fn main() {

    let port = 8080;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    let num_threads = num_cpus::get();
    let pool = ThreadPool::new(num_threads);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
    
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to unwrap stream: {}", e);
            }
        }
    }
}
fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    match buf_reader.read_line(&mut request_line) {
        Ok(_) => {
            let pages = vec![
                ("/", "pages/index.html"),
                ("/404", "pages/404.html"), // Add new page and corresponding URL
            ];
            
            let (status_line, filename) = pages.iter().find(|(path, _)| request_line.trim().starts_with(&format!("GET {} HTTP/1.1", path)))
                .unwrap_or(&("/404", "pages/404.html"));

            match fs::read_to_string(filename) {
                Ok(contents) => {
                    let length = contents.len();

                    let response =
                        format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}", status_line, length, contents);

                    match stream.write(response.as_bytes()) {
                        Ok(_) => {
                            match stream.flush() {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to flush stream: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read request line: {}", e);
        }
    }

}