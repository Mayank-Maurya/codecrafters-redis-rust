#![allow(unused_imports)]
use std::{env, net::TcpListener};
use std::io::{self, Write};

fn main() {

    let stdin = io::stdin();
    let mut input;
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    print!("$ ");
    io::stdout().flush().unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                // take input
                input = String::new();
                stdin.read_line(&mut input).unwrap();
                let commands: Vec<&str> = input.split_ascii_whitespace().collect();
                // let mut response;

                // match commands[1] {
                //     "PING" => response = "+PONG\r\n",
                //     _ => response = "unknown command",
                // }

                let _err = _stream.write(b"+PONG\r\n");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
