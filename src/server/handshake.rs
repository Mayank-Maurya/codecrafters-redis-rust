use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::codec::encoder::encode_array;

pub async fn handshake(mut stream: TcpStream) {
    // send PING command
    let value: String;
    let mut args = Vec::new();
    args.push("PING".to_string());
    let response =encode_array(args);
    if let Err(e) = stream.write_all(&response).await {
        eprintln!("Failed to write to stream: {}", e);
    }
    println!("sends command PING");
}