use std::error::Error;
use std::io;

use bytes::Buf;
use bytes::BytesMut;
use tokio::net::TcpStream;

async fn handle_client(stream: TcpStream) {
    stream.readable().await.unwrap();
    println!("readata");
    let mut data2 = BytesMut::with_capacity(4096);
    loop {
        match stream.try_read_buf(&mut data2) {
            Ok(n) => {
                println!("read {} bytes", n);
                println!("remaining: {}", data2.remaining());
                let value = data2.get_f32();
                println!("value: {}", value);

                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("localhost:12413").await?;
    let handle = tokio::spawn(async move {
        handle_client(stream).await;
    });

    handle.await?;
    println!("owari");
    Ok(())
}
