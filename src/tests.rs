#[cfg(test)]
mod tests {
    use crate::*;
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
