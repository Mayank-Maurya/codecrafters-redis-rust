use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::codec::encoder::encode_array;

pub async fn handshake_1(stream: TcpStream) -> TcpStream {
    // send PING command
    let mut args = Vec::new();
    args.push("PING".to_string());
    let response = encode_array(args);
    println!("sends command PING");
    send_command(stream,response).await
}

pub fn handshake_2(port: String) -> Vec<u8> {
    // send PING command
    let mut args = Vec::new();
    args.push("REPLCONF".to_string());
    args.push("listening-port".to_string());
    args.push(port);
    return encode_array(args);
}

pub fn handshake_3() -> Vec<u8> {
    // send PING command
    let mut args = Vec::new();
    args.push("REPLCONF".to_string());
    args.push("capa".to_string());
    args.push("psync2".to_string());
    println!("sends command ReplConf psync2");
    return encode_array(args);
    // send_command(stream,response).await
}

pub async fn send_command(mut stream: TcpStream, messages: Vec<u8>) -> TcpStream {
    if let Err(e) = stream.write_all(&messages).await {
        eprintln!("Failed to write to stream: {}", e);
    }
    return stream;
}