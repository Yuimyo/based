use core::panic;
use std::{
    io::{Cursor, Error},
    thread, time,
};

use bytes::{Buf, BufMut, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

async fn handle_client(stream: TcpStream) {
    println!("enter.");

    let mut stream = TcpConnection::new(stream);
    let bytes = Vec::from([2u8, 135u8, 34u8, 23u8, 24u8]);
    println!("ha?");
    let len = stream.write_bytes(bytes.as_slice()).await.unwrap();
    println!("tataha? {}", len);
    thread::sleep(time::Duration::from_secs(5));

    println!("exit.");
}

// bytesをparseする(byte列をどこでぶつ切りにするか)部分と、Processingする（パケットの応答など）部分がある。
// 実装的には単位的で、どこでどのように実装するかを考える必要がある。
//
// Testを書くことも考えたら、Processingにbytesのモックを入れたいので、Processingは分離したいね。
// Processing部分に、parseに必要なbytes長を実装させたい。
// これがbytesのモックを同一にして、bytes長が一致するかで調べる。
//
// parseする部分については、パケットの終端点に関するプロトコルが統一されていれば、1つの機構で処理できる可能性がある。
// プロトコルが異なれば、各々で実装せざるを得ない。
// →統一パターンと非統一パターンの2つを実装してみる必要がありそう。

pub trait PacketSplitter {
    async fn read_packet(&mut self) -> Result<Vec<u8>, Error>;
    async fn write_bytes(&mut self, src: &[u8]) -> Result<u64, Error>;
}

pub trait Processor<ProcessResult> {
    fn process_bytes(&mut self, buf: &[u8]) -> ProcessResult;
}

// pub trait Processor<T> {

//     // fn check (&mut self, buf: &[u8], start_index: usize) -> Result<u64, Error>
//     // fn check (&mut self, buf: &Vec<u8>, start_index: usize) -> Result<u64, Error>
//     //   as_ref()
//     // fn check (&mut self, buf: &mut Cursor<&[u8]>) -> Result<u64, Error>
//     // fn check (&mut self, buf: &Bytes) -> Result<u64, Error>
//     fn check(&mut self, buf: &[u8]) -> Result<u64, Error> {
//         self.check_by_idx(buf, 0)
//     }

//     fn check_by_idx(&mut self, buf: &[u8], start_index: u64) -> Result<u64, Error> {
//         let mut buf = Cursor::new(buf);
//         buf.set_position(start_index);
//         match self.parse(&mut buf) {
//             Ok(_) => {
//                 let len: u64 = buf.position() - start_index;

//                 buf.set_position(start_index);
//                 Ok(len)
//             }
//             Err(e) => {
//                 buf.set_position(start_index);
//                 Err(e)
//             }
//         }
//     }

//     fn parse(&mut self, buf: &mut Cursor<&[u8]>) -> Result<Hogey, Error>;

// }
pub struct Hogey {
    data: u32,
}

pub struct Handshake {}

pub struct PacketState {}

impl Handshake {
    pub fn new() -> Self {
        Handshake {}
    }

    pub fn process_bytes() {}
}

pub struct TcpConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl PacketSplitter for TcpConnection {
    async fn read_packet(&mut self) -> Result<Vec<u8>, Error> {
        self.load_buf().await?;
        let mut cur = Cursor::new(&mut self.buffer);
        if 4 > cur.remaining() {
            return Err(Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        let len = cur.get_u32() as usize;

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
        let size = self.stream.read_buf(&mut self.buffer).await?;

        Ok(size)
    }

    // fn parse_frame(&mut self) -> Result<Option<McPacketFrame>, Error> {
    //     let mut buf = Cursor::new(&self.buffer[..]);

    //     match McPacketFrame::check(&mut buf) {
    //         Ok(_) => {
    //             let len = buf.position() as usize;

    //             buf.set_position(0);
    //             let frame = McPacketFrame::parse(&mut buf).unwrap();

    //             self.buffer.advance(len);
    //             Ok(Some(frame))
    //         }
    //         Err(FrameError::Incomplete) => Ok(None),
    //         Err(e) => Err(e.into()),
    //     }
    // }

    // pub async fn read_frame(&mut self) -> Result<(), Error> {
    //     loop {
    //         if let Some(frame) = self.parse_frame()? {
    //             return Ok(Some(frame));
    //         }
    //         let a = self.stream.read_buf(&mut self.buffer).await.unwrap();
    //         println!("{}", &(a));
    //         if a == 0 {
    //             if self.buffer.is_empty() {
    //                 return Ok(None);
    //             } else {
    //                 return Err("connection reset by peer".into());
    //             }
    //         }
    //     }
    // }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:34254").await?;

    while let Ok((stream, addr)) = listener.accept().await {
        handle_client(stream).await;
    }

    println!("Hello, world!");
    Ok(())
}
