#[cfg(test)]
mod tests;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};
pub fn read_request(stream: &TcpStream) -> Vec<String> {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .filter_map(|result| result.ok())
        .take_while(|line| !line.is_empty())
        .collect();
    http_request
}
pub fn parse_request<'a>(http_request: Vec<String>) -> (&'a str, &'a str) {
    let request_line = &http_request[0];
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    (status_line, filename)
}

pub fn handle_connection(mut stream: TcpStream) -> Result<(), std::io::Error> {
    println!("Connection established from: {:?}", stream.peer_addr()?);
    let http_request = read_request(&stream);
    if http_request.is_empty() {
        return Ok(());
    };
    let (status_line, filename) = parse_request(http_request);

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush()?;
    Ok(())
}
pub fn run_server(listner: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    for streams in listner.incoming() {
        let stream = streams?;
        thread::spawn(move || -> Result<(), std::io::Error> {
            handle_connection(stream)?;
            Ok(())
        });
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listner = TcpListener::bind("127.0.0.1:7878").unwrap();
    run_server(listner)
}
