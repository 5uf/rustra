use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use rustra::ThreadPool;

fn main() {
    let port = 8080;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|e| panic!("Failed to bind to port {}: {}", port, e));

    // num_cpus::get() returns the logical CPU count; guard against the
    // (theoretical) case where it returns 0 to prevent a ThreadPool panic.
    let num_threads = num_cpus::get().max(1);
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

/// Route table: each entry is `(request path, HTTP status text, HTML file)`.
/// Add new routes here; unmatched requests fall back to [`NOT_FOUND`].
const ROUTES: &[(&str, &str, &str)] = &[
    ("/", "200 OK", "pages/index.html"),
];

const NOT_FOUND: (&str, &str, &str) = ("", "404 Not Found", "pages/404.html");

/// Find the route that matches the first line of an HTTP request.
///
/// Returns `(path, status_text, html_file)`.  An empty `path` in the returned
/// tuple indicates the 404 fallback was selected.
pub fn find_route(request_line: &str) -> (&'static str, &'static str, &'static str) {
    ROUTES
        .iter()
        .find(|(path, _, _)| {
            request_line
                .trim()
                .starts_with(&format!("GET {} HTTP/1.1", path))
        })
        .copied()
        .unwrap_or(NOT_FOUND)
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    match buf_reader.read_line(&mut request_line) {
        Ok(_) => {
            let (_, status, filename) = find_route(&request_line);

            match fs::read_to_string(filename) {
                Ok(contents) => {
                    let response = format!(
                        "HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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
                    // Send a minimal 500 instead of silently dropping the connection.
                    let body = "<html><body><h1>500 Internal Server Error</h1></body></html>";
                    let response = format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read request line: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_route_root_returns_200() {
        let (_, status, filename) = find_route("GET / HTTP/1.1");
        assert_eq!(status, "200 OK");
        assert_eq!(filename, "pages/index.html");
    }

    #[test]
    fn find_route_unknown_path_returns_404() {
        let (_, status, filename) = find_route("GET /nonexistent HTTP/1.1");
        assert_eq!(status, "404 Not Found");
        assert_eq!(filename, "pages/404.html");
    }

    #[test]
    fn find_route_non_get_method_returns_404() {
        let (_, status, _) = find_route("POST / HTTP/1.1");
        assert_eq!(status, "404 Not Found");
    }

    #[test]
    fn find_route_handles_crlf_line_ending() {
        let (_, status, _) = find_route("GET / HTTP/1.1\r\n");
        assert_eq!(status, "200 OK");
    }

    #[test]
    fn find_route_does_not_match_subpath_as_root() {
        // "/about" must not match the "/" route.
        let (_, status, _) = find_route("GET /about HTTP/1.1");
        assert_eq!(status, "404 Not Found");
    }
}
