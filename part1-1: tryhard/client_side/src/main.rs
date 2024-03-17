use core::panic;
use std::io::{Cursor, Error};

use bytes::{Buf, BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

pub trait PacketSplitter {
    fn read_packet(&mut self) -> impl std::future::Future<Output = Result<Vec<u8>, Error>> + Send;
    fn write_bytes(
        &mut self,
        src: &[u8],
    ) -> impl std::future::Future<Output = Result<u64, Error>> + Send;
}

pub trait Processor<ProcessResult> {
    fn process_bytes(&mut self, buf: &[u8]) -> ProcessResult;
}

pub struct TcpConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl PacketSplitter for TcpConnection {
    async fn read_packet(&mut self) -> Result<Vec<u8>, Error> {
        self.load_buf().await?;
        println!("ffffffff {}", self.buffer.remaining());

        let mut cur = Cursor::new(&mut self.buffer);
        if 4 > cur.remaining() {
            return Err(Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        let len = cur.get_u32() as usize;
        println!("ffffffff {}", len);

        if len > cur.remaining() {
            return Err(Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        let mut data = vec![0; len];
        cur.read_exact(&mut data).await?;
        self.buffer.advance(4 + len);

        Ok(data)
    }

    async fn write_bytes(&mut self, src: &[u8]) -> Result<u64, Error> {
        self.stream.write_u32(src.len() as u32).await?;
        self.stream.write_all(src).await?;
        self.stream.flush().await?;
        Ok(src.len() as u64)
    }
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        TcpConnection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    async fn load_buf(&mut self) -> Result<usize, Error> {
        let mut size_total = 0;
        loop {
            println!("ff {}", size_total);
            self.stream.get_ref().readable().await?;
            match self.stream.get_ref().try_read_buf(&mut self.buffer) {
                Ok(0) => break,
                Ok(n) => {
                    size_total += n;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(size_total)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("localhost:34254").await?;

    let mut stream = TcpConnection::new(stream);

    let data = stream.read_packet().await.unwrap();
    for i in 0..data.len() {
        print!(", {}", data[i]);
    }
    println!("Hello, world!");
    Ok(())
}
