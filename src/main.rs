#![allow(unused_imports)]
use std::{env, net::TcpListener};
use std::io::{self, Write};

fn main() {

    let stdin = io::stdin();
    let mut input;
    let listener = TcpListener::bind("127.0.0.1:6380").unwrap();
    print!("$ ");
    io::stdout().flush().unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                // take input
                input = String::new();
                stdin.read_line(&mut input).unwrap();
                let commands: Vec<&str> = input.split_ascii_whitespace().collect();

                match commands[1] {
                    "PING" => println!("PONG"),
                    _ => println!("unknown command"),
                }

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
