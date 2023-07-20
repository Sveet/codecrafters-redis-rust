use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let response = "+PONG\r\n".as_bytes();
                let _ = _stream.write(response);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
