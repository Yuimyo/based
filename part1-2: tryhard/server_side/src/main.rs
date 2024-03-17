use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn handle_connection(mut stream: TcpStream) {
    stream.writable().await.unwrap();
    stream.write_f32(141f32).await.unwrap();
    stream.flush().await.unwrap();
    println!("owari");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:12413").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        handle_connection(stream).await;
    }

    Ok(())
}
