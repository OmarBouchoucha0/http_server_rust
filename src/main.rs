use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    for streams in listener.incoming() {
        let stream = streams.unwrap();
        println!("Connection Established");
    }
}
