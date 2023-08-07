use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let mut storage = HashMap::<String, String>::new();
    let mut clients: Vec<TcpStream> = vec![];
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking for listener");

    loop {
        match listener.accept() {
            Ok(_stream) => {
                println!("accepted new connection");
                _stream
                    .0
                    .set_nonblocking(true)
                    .expect("Cannot set non-blocking for socket");
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
            let mut buf = vec![0; 255];
            match client.read(&mut buf) {
                Ok(_) => {
                    let message = String::from_utf8(buf).unwrap();
                    for command in parse_message(message) {
                        match command.keyword {
                            ReservedKeys::PING => {
                                let _ = client.write("+PONG\r\n".as_bytes());
                            }
                            ReservedKeys::ECHO => {
                                if let Some(resp) = command.args.first() {
                                    let response = format!("+{resp}\r\n");
                                    let _ = client.write(response.as_bytes());
                                }
                            }
                            ReservedKeys::SET => {
                                let key = command.args[0].to_owned();
                                let value = command.args[1].to_owned();
                                storage.insert(key, value);
                                let _ = client.write(format!("+OK\r\n").as_bytes());
                            }
                            ReservedKeys::GET => {
                                let key = command.args[0].to_owned();
                                let value = storage.get(&key);
                                if let Some(value) = value {
                                    let resp = format!("+{value}\r\n");
                                    let _ = client
                                        .write(resp.as_bytes())
                                        .expect("couldn't write response");
                                }
                            }
                            _ => {}
                        }
                    }
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
struct Command {
    keyword: ReservedKeys,
    args: Vec<String>,
}
enum ReservedKeys {
    ECHO,
    PING,
    UNKNOWN,
    SET,
    GET,
}

fn parse_message(message: String) -> Vec<Command> {
    let mut chunks = message.split("\r\n");
    let mut commands = Vec::<Command>::new();
    let mut arr_len: u32 = 1;
    let mut parsed_keyword = false;

    loop {
        let mut chunk = match chunks.next() {
            None => break,
            Some(chunk) => chunk,
        };
        let mut word: Option<&str> = None;
        let mut keyword = ReservedKeys::UNKNOWN;
        let mut args = Vec::<String>::new();

        while arr_len > 0 {
            let first_char = match chunk.chars().next() {
                Some(c) => c,
                None => break,
            };
            match first_char {
                '*' => {
                    arr_len = parse_number(chunk, '*') + 1;
                }
                '$' => {
                    let _ = parse_number(chunk, '$');
                    word = chunks.next();
                }
                _ => word = Some(chunk),
            }

            if word.is_some() {
                if !parsed_keyword {
                    let w = word.unwrap().to_uppercase();
                    keyword = match w.as_str() {
                        "PING" => ReservedKeys::PING,
                        "ECHO" => ReservedKeys::ECHO,
                        "SET" => ReservedKeys::ECHO,
                        "GET" => ReservedKeys::ECHO,
                        _ => ReservedKeys::UNKNOWN,
                    };
                    parsed_keyword = true;
                } else {
                    if let Some(word) = word {
                        args.push(word.to_string());
                    }
                }
            }

            chunk = match chunks.next() {
                None => break,
                Some(c) => c,
            };
            arr_len -= 1;
        }
        let command = Command { keyword, args };
        commands.push(command);
    }
    return commands;
}

fn parse_number(chunk: &str, key: char) -> u32 {
    return chunk
        .chars()
        .nth(1)
        .expect(format!("expected value to follow {:?}", key).as_str())
        .to_digit(10)
        .expect(format!("expected numeric value to follow {:?}", key).as_str());
}
