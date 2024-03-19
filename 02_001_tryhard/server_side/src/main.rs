use bytes::{Buf, BufMut, Bytes, BytesMut};
use pktlib::{
    connection::ConnectionMode,
    handshake::{Handshake, HandshakeState},
    packet::PacketProcessor,
};
use std::{error::Error, io::Cursor};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

fn print_recv_bytes(recv_bytes: &[u8]) {
    let mut recv_buf = Cursor::new(recv_bytes);
    let mut recv_values = Vec::<u32>::new();
    while recv_buf.remaining() > 0 {
        recv_values.push(recv_buf.get_u32());
    }
    println!("{:?}", recv_values);
}

async fn process_socket(socket: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("started.");

    let mut socket: BufWriter<TcpStream> = BufWriter::new(socket);
    let mut packet_processor = PacketProcessor::new();
    let mut buf = Bytes::new();

    let mut handshake = Handshake::new(ConnectionMode::Server);

    loop {
        if handshake.required_to_read() {
            while !packet_processor.has_packet() {
                let mut buf = BytesMut::with_capacity(512);
                socket.get_ref().readable().await?;
                match socket.get_ref().try_read_buf(&mut buf) {
                    Ok(_) => {
                        packet_processor.put(&buf[..]);
                        break;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            let packet = packet_processor.pop().unwrap();
            print_recv_bytes(&packet[..]);
            buf = Bytes::from(packet);
        }

        match handshake.communicate(&buf) {
            HandshakeState::InProgress { send_bytes } => {
                if !send_bytes.is_empty() {
                    socket.get_ref().writable().await?;
                    PacketProcessor::format_and_write(&mut socket, &send_bytes[..]).await?;
                    socket.flush().await?;
                }
            }
            HandshakeState::Finished { send_bytes } => {
                if !send_bytes.is_empty() {
                    socket.get_ref().writable().await?;
                    PacketProcessor::format_and_write(&mut socket, &send_bytes[..]).await?;
                    socket.flush().await?;
                }
                break;
            }
            HandshakeState::Nothing => todo!(),
            HandshakeState::Error(e) => {
                return Err(e);
            }
        }
    }

    println!("ended.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:13132").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await?;
    }
}
