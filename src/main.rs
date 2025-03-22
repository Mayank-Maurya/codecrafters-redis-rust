#![allow(unused_imports)]
use std::rc::Rc;
use std::io::{self, BufRead, Read, Write};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 1024];
    
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => {
                        return
                    },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
    
                if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
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
