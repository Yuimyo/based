use bytes::{Buf, BufMut, BytesMut};
use std::{error::Error, io::Cursor};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

struct Handshake {}
impl Handshake {
    pub fn new() -> Self {
        Handshake {}
    }

    pub fn begin(&mut self) -> Vec<u8> {
        let mut send_buf = BytesMut::with_capacity(4096);

        send_buf.split().freeze().to_vec()
    }

    pub fn communicate(&mut self, recv_bytes: &[u8]) -> HandshakeState {
        let mut send_buf = BytesMut::with_capacity(4096);

        // 仮: recv_bytesの表示
        let mut recv_buf = Cursor::new(recv_bytes);
        let mut recv_values = Vec::<u32>::new();
        while recv_buf.remaining() > 0 {
            recv_values.push(recv_buf.get_u32());
        }
        println!("{:?}", recv_values);

        // 仮: 仮の値
        send_buf.put_u32(67);
        send_buf.put_u32(89);

        let send_bytes = send_buf.split().freeze().to_vec();
        HandshakeState::Finished { send_bytes }
    }
}

enum HandshakeState {
    Nothing,
    InProgress { send_bytes: Vec<u8> },
    Finished { send_bytes: Vec<u8> },
    Error(Box<dyn Error>),
}

async fn process_socket(mut socket: TcpStream) {
    println!("started.");

    let mut buf = BytesMut::with_capacity(4096);

    let mut handshake = Handshake::new();

    loop {
        socket.readable().await.unwrap();
        socket.try_read_buf(&mut buf).unwrap();

        match handshake.communicate(&buf.split().freeze()[..]) {
            HandshakeState::InProgress { send_bytes } => {}
            HandshakeState::Finished { send_bytes } => {
                if !send_bytes.is_empty() {
                    socket.writable().await.unwrap();
                    socket.try_write(&send_bytes[..]).unwrap();
                    socket.flush().await.unwrap();
                }
                println!("finished.");
                break;
            }
            HandshakeState::Nothing => todo!(),
            HandshakeState::Error(e) => todo!(),
        }
    }

    println!("ended.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:13131").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await;
    }
}
