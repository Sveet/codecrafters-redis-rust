use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let mut clients: Vec<TcpStream> = vec![];
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking on listener");

    loop {
        match listener.accept() {
            Ok(_stream) => {
                println!("accepted new connection");
                _stream
                    .0
                    .set_nonblocking(true)
                    .expect("Cannot set non-blocking on socket");
                clients.push(_stream.0);
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    println!("error: {}", e);
                }
            }
        }

        let mut disconnected: Vec<usize> = vec![];
        for (i, client) in clients.iter_mut().enumerate() {
            let mut buf = [0; 512];
            match client.read(&mut buf) {
                Ok(_) => {
                    handle_stream(client);
                }
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        continue;
                    } else {
                        println!("error: {}", e);
                        disconnected.push(i);
                    }
                }
            }
        }

        for &index in disconnected.iter().rev() {
            clients.remove(index);
        }
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let response = "+PONG\r\n".as_bytes();
    let _ = stream.write(response);
}
