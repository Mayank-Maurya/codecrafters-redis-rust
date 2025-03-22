#![allow(unused_imports)]
use std::{env, net::TcpListener};
use std::io::{self, BufRead, BufReader, Read, Write};

fn main() {

    let stdin = io::stdin();
    // let mut input;
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {

                let mut reader = BufReader::new(&stream);
                let mut input = String::new();

                match reader.read_line(&mut input) {
                    Ok(_) => {
                        stream.write(b"+PONG\r\n");
                    },
                    Err(e) =>  {
                        println!("{}", e);
                    }
                }

                // let _err = stream.write(b"+PONG\r\n");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
