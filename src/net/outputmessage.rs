use crate::net::networkmessage::NetworkMessage;
use std::sync::{Arc, Mutex};
use crate::net::tools::adler32;

pub struct OutputMessagePool;

pub struct OutputMessage {
    pub msg: NetworkMessage,
    pub output_buffer_start: usize,
}

impl OutputMessage {
    pub fn new() -> Self {
        Self {
            msg: NetworkMessage::new(),
            output_buffer_start: NetworkMessage::INITIAL_BUFFER_POSITION as usize,
        }
    }

    pub fn get_output_buffer(&self) -> &[u8] {
        &self.msg.buffer[self.output_buffer_start..self.output_buffer_start + self.msg.length as usize]
    }

    pub fn write_message_length(&mut self) {
        let length = self.msg.length;
        self.add_header(length);
    }

    pub fn add_crypto_header(&mut self, add_checksum: bool) {
        if add_checksum {
            let checksum = adler32(
                &self.msg.buffer[self.output_buffer_start..self.output_buffer_start + self.msg.length as usize],
            );
            self.add_header(checksum);
        }
        self.write_message_length();
    }

    pub fn append(&mut self, other: &NetworkMessage) {
        let msg_len = other.length;
        let src = &other.buffer[NetworkMessage::INITIAL_BUFFER_POSITION as usize..NetworkMessage::INITIAL_BUFFER_POSITION as usize + msg_len as usize];
        let dest_pos = self.msg.position as usize;
        self.msg.buffer[dest_pos..dest_pos + msg_len as usize].copy_from_slice(src);
        self.msg.length += msg_len;
        self.msg.position += msg_len;
    }

    fn add_header<T: Copy>(&mut self, value: T) {
        let size = std::mem::size_of::<T>();
        assert!(self.output_buffer_start >= size);
        self.output_buffer_start -= size;
        let ptr = &value as *const T as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
        self.msg.buffer[self.output_buffer_start..self.output_buffer_start + size].copy_from_slice(slice);
        self.msg.length += size as u16;
    }
}


impl OutputMessagePool {
    pub fn get_output_message() -> Arc<Mutex<OutputMessage>> {
        Arc::new(Mutex::new(OutputMessage::new()))
    }
}
