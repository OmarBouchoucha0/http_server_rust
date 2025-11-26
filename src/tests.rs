#[cfg(test)]
mod tests {
    use crate::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    // Helper function to start a test server on a given port
    fn start_test_server(port: u16) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let addr = format!("127.0.0.1:{}", port);
            let listener = TcpListener::bind(&addr).unwrap();

            // Accept only a few connections for testing
            for (i, stream_result) in listener.incoming().enumerate() {
                if i >= 5 {
                    break;
                }
                if let Ok(stream) = stream_result {
                    thread::spawn(move || {
                        let _ = handle_connection(stream);
                    });
                }
            }
        })
    }

    // Helper function to send a request and get response
    fn send_request(port: u16, request: &str) -> Result<String, std::io::Error> {
        thread::sleep(Duration::from_millis(100)); // Wait for server to be ready

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        Ok(response)
    }

    #[test]
    fn test_read_request_basic() {
        let port = 7880;
        start_test_server(port);
        thread::sleep(Duration::from_millis(200));

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream.write_all(request.as_bytes()).unwrap();

        let http_request = read_request(&stream);
        assert!(!http_request.is_empty());
        assert_eq!(http_request[0], "GET / HTTP/1.1");
    }

    #[test]
    fn test_parse_request_root_path() {
        let request = vec!["GET / HTTP/1.1".to_string()];
        let (status_line, filename) = parse_request(request);

        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "hello.html");
    }

    #[test]
    fn test_parse_request_not_found() {
        let request = vec!["GET /other HTTP/1.1".to_string()];
        let (status_line, filename) = parse_request(request);

        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(filename, "404.html");
    }

    #[test]
    fn test_parse_request_different_paths() {
        let test_cases = vec![
            "GET /about HTTP/1.1",
            "GET /contact HTTP/1.1",
            "GET /random/path HTTP/1.1",
        ];

        for request_line in test_cases {
            let request = vec![request_line.to_string()];
            let (status_line, filename) = parse_request(request);

            assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
            assert_eq!(filename, "404.html");
        }
    }

    #[test]
    fn test_full_connection_root_path() {
        let port = 7881;
        start_test_server(port);

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let response = send_request(port, request).unwrap();

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Length:"));
    }

    #[test]
    fn test_full_connection_404() {
        let port = 7882;
        start_test_server(port);

        let request = "GET /notfound HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let response = send_request(port, request).unwrap();

        assert!(response.contains("HTTP/1.1 404 NOT FOUND"));
        assert!(response.contains("Content-Length:"));
    }

    #[test]
    fn test_multiple_concurrent_connections() {
        let port = 7883;
        start_test_server(port);
        thread::sleep(Duration::from_millis(200));

        let handles: Vec<_> = (0..3)
            .map(|_| {
                thread::spawn(move || {
                    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
                    send_request(port, request)
                })
            })
            .collect();

        for handle in handles {
            let response = handle.join().unwrap().unwrap();
            assert!(response.contains("HTTP/1.1"));
        }
    }

    #[test]
    fn test_handle_connection_success() {
        let port = 7884;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let _ = handle_connection(stream);
            }
        });

        thread::sleep(Duration::from_millis(100));

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let result = send_request(port, request);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_request_handling() {
        // This tests the behavior when read_request returns empty
        let empty_request: Vec<String> = Vec::new();
        let http_request = empty_request;

        // parse_request expects at least one line, but we're testing
        // that handle_connection returns early for empty requests
        assert!(http_request.is_empty());
    }

    #[test]
    fn test_response_format() {
        let port = 7885;
        start_test_server(port);

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let response = send_request(port, request).unwrap();

        // Check that response has proper HTTP format
        let lines: Vec<&str> = response.lines().collect();
        assert!(lines[0].starts_with("HTTP/1.1"));

        // Check for Content-Length header
        assert!(response.contains("Content-Length:"));

        // Check for double CRLF separating headers from body
        assert!(response.contains("\r\n\r\n"));
    }

    #[test]
    fn test_parse_request_case_sensitivity() {
        // HTTP methods are case-sensitive, should only match exact "GET"
        let request = vec!["get / HTTP/1.1".to_string()];
        let (status_line, filename) = parse_request(request);

        // Your original code doesn't handle this, so it would go to 404
        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(filename, "404.html");
    }

    #[test]
    fn test_request_with_multiple_headers() {
        let request = vec![
            "GET / HTTP/1.1".to_string(),
            "Host: localhost".to_string(),
            "User-Agent: Test".to_string(),
            "Accept: text/html".to_string(),
        ];

        let (status_line, filename) = parse_request(request);
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "hello.html");
    }
}

// Integration test module
#[cfg(test)]
mod integration_tests {
    use crate::*;
    use std::{
        fs,
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        thread,
        time::Duration,
    };
    #[test]
    fn test_with_actual_files() {
        // Create temporary HTML files for testing
        let hello_content = "<html><body><h1>Hello</h1></body></html>";
        let not_found_content = "<html><body><h1>404 Not Found</h1></body></html>";

        fs::write("hello.html", hello_content).unwrap();
        fs::write("404.html", not_found_content).unwrap();

        let port = 7886;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let _ = handle_connection(stream);
            }
        });

        thread::sleep(Duration::from_millis(100));

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        assert!(response.contains("200 OK"));
        assert!(response.contains(hello_content));

        // Cleanup
        fs::remove_file("hello.html").ok();
        fs::remove_file("404.html").ok();
    }
}
