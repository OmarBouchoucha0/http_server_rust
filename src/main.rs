use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> Result<(), std::io::Error> {
    println!("Connection established from: {:?}", stream.peer_addr()?);
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .filter_map(|result| result.ok())
        .take_while(|line| !line.is_empty())
        .collect();

    if http_request.is_empty() {
        return Ok(());
    }
    println!("Request: {http_request:?}");
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("hello.html")?;
    let contents_length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {contents_length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    for streams in listener.incoming() {
        let stream = streams?;
        handle_connection(stream)?;
    }
    Ok(())
}
