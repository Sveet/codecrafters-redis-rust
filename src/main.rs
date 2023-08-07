use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");

                let mut buf = [0; 512];
                loop {
                    let read_count = _stream
                        .read(&mut buf)
                        .expect("Error reading stream into buffer");
                    if read_count == 0 {
                        break;
                    }
                    handle_stream(&mut _stream);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let response = "+PONG\r\n".as_bytes();
    let _ = stream.write(response);
}
