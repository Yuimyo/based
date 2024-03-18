use std::error::Error;

use bytes::{BufMut, BytesMut};

use crate::connection::ConnectionMode;

pub struct Handshake {
    connection_mode: ConnectionMode,
    pkt_cnt: u32,
}
impl Handshake {
    pub fn new(connection_mode: ConnectionMode) -> Self {
        Handshake {
            connection_mode,
            pkt_cnt: 0,
        }
    }

    pub fn required_to_read(&mut self) -> bool {
        if self.pkt_cnt == 0 && self.connection_mode == ConnectionMode::Client {
            return false;
        }
        true
    }

    pub fn communicate(&mut self, recv_bytes: &[u8]) -> HandshakeState {
        self.pkt_cnt += 1;

        match self.connection_mode {
            ConnectionMode::Server => match self.pkt_cnt {
                1 => {
                    let mut send_buf = BytesMut::with_capacity(4096);
                    send_buf.put_u32(67);
                    send_buf.put_u32(89);

                    HandshakeState::Finished {
                        send_bytes: send_buf.split().freeze().to_vec(),
                    }
                }
                _ => todo!(),
            },
            ConnectionMode::Client => match self.pkt_cnt {
                1 => {
                    let mut send_buf = BytesMut::with_capacity(4096);
                    send_buf.put_u32(123);
                    send_buf.put_u32(45);

                    HandshakeState::InProgress {
                        send_bytes: send_buf.split().freeze().to_vec(),
                    }
                }
                2 => HandshakeState::Finished {
                    send_bytes: vec![0; 0],
                },
                _ => todo!(),
            },
        }
    }
}

pub enum HandshakeState {
    Nothing,
    InProgress { send_bytes: Vec<u8> },
    Finished { send_bytes: Vec<u8> },
    Error(Box<dyn Error>),
}
