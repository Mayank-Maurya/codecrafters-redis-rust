#![allow(unused_imports)]
use std::rc::Rc;
use std::{env};
use std::io::{self, BufRead, Read, Write};
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            loop { 
                let mut reader = BufReader::new(&mut socket);
                let mut input = String::new();
                input.clear();

                match reader.read_line(&mut input).await {
                    Ok(0) => {
                        break;
                    },
                    Ok(_) => {
                        socket.write(b"+PONG\r\n");
                    },
                    Err(e) =>  {
                        println!("{}", e);
                    }
                }

            }
        });
    }

    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(mut stream) => {
    //         loop {
    //             let mut reader = BufReader::new(&stream);
    //             let mut input: String = String::new();

    //             match reader.read_line(&mut input) {
    //                 Ok(0) => {
    //                     break;
    //                 },
    //                 Ok(_) => {
    //                     stream.write(b"+PONG\r\n");
    //                 },
    //                 Err(e) =>  {
    //                     println!("{}", e);
    //                 }
    //             }
    //         }

    //             // let _err = stream.write(b"+PONG\r\n");
    //         }
    //         Err(e) => {
    //             println!("error: {}", e);
    //         }
    //     }
    // }
}
