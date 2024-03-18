use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{cmp::min, collections::VecDeque, io::Cursor};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct PacketProcessor {
    pkt_queue: VecDeque<Vec<u8>>,
    buf: BytesMut,
}
impl PacketProcessor {
    pub fn new() -> Self {
        PacketProcessor {
            pkt_queue: VecDeque::<Vec<u8>>::new(),
            buf: BytesMut::with_capacity(0),
        }
    }

    pub fn put(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);

        if let Some(packet) = self.try_to_packet() {
            self.pkt_queue.push_back(packet);
        };
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        if self.has_packet() {
            return Some(self.pkt_queue.pop_front().unwrap());
        }
        None
    }

    pub fn has_packet(&self) -> bool {
        self.count() > 0
    }

    pub fn count(&self) -> usize {
        self.pkt_queue.len()
    }

    fn try_to_packet(&mut self) -> Option<Vec<u8>> {
        if self.buf.remaining() >= 4 {
            let length = Cursor::new(&(self.buf)[..4]).get_u32() as usize;

            if self.buf.remaining() >= 4 + length {
                self.buf.advance(4);
                let packet = self.buf.split_to(length).to_vec();
                return Some(packet);
            }
        }
        None
    }

    pub fn format(data: &[u8]) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u32(data.len() as u32);
        buf.extend_from_slice(data);
        buf.freeze()
    }
}

#[cfg(test)]
mod packet_processor {
    use super::*;

    #[test]
    fn process_packet() {
        let mut pp = PacketProcessor::new();

        let length: u32 = 4;

        let mut output = Vec::<u8>::new();
        output.extend_from_slice(&length.to_be_bytes());
        pp.put(&output);
        pp.put(&vec![4u8, 3u8, 1u8]);
        assert_eq!(pp.count(), 0);

        pp.put(&vec![72u8, 0u8, 0u8, 0u8, 5u8, 23u8]);
        assert_eq!(pp.count(), 1);

        assert_eq!(pp.pop().unwrap(), vec![4u8, 3u8, 1u8, 72u8]);
        assert_eq!(pp.count(), 0);

        pp.put(&vec![72u8, 0u8, 23u8, 0u8, 0u8, 0u8]);
        assert_eq!(pp.count(), 1);
        assert_eq!(pp.buf.len(), 2);
    }

    #[test]
    fn format_data() {
        let data = [72u8, 0u8];
        let actual = PacketProcessor::format(&data);
        assert_eq!(actual, vec![0u8, 0u8, 0u8, 2u8, 72u8, 0u8]);
    }

    // packet長にとんでもなく大きい値がきたときに弾いとくと安全な気もする。
    // #[test]
    fn avoid_huge_length() {
        todo!()
    }
}
