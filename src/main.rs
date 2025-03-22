#![allow(unused_imports)]
use std::rc::Rc;
use std::{env, net::TcpListener};
use std::io::{self, BufRead, BufReader, Read, Write};

fn main() {

    let stdin = io::stdin();
    // let mut input;
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut input: String = String::new();

                match reader.read_line(&mut input) {
                    Ok(0) => {
                        break;
                    },
                    Ok(_) => {
                        stream.write_all(b"+PONG\r\n");
                    },
                    Err(e) =>  {
                        println!("{}", e);
                    }
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
