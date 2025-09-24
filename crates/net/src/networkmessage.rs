// src/networkmessage.rs
// Port av TFS NetworkMessage till Rust

use crate::consts::*;
use std::str;


pub type MsgSize = u16;

pub struct NetworkMessage {
    pub length: MsgSize,
    pub position: MsgSize,
    pub overrun: bool,
    pub buffer: Vec<u8>,
}

impl NetworkMessage {
    pub const INITIAL_BUFFER_POSITION: MsgSize = 8;
    pub const HEADER_LENGTH: usize = 2;
    pub const CHECKSUM_LENGTH: usize = 4;
    pub const XTEA_MULTIPLE: usize = 8;
    pub const MAX_BODY_LENGTH: usize =
        (NETWORKMESSAGE_MAXSIZE as usize) - Self::HEADER_LENGTH - Self::CHECKSUM_LENGTH - Self::XTEA_MULTIPLE;
    pub const MAX_PROTOCOL_BODY_LENGTH: usize = Self::MAX_BODY_LENGTH - 10;

    pub fn new() -> Self {
        Self {
            length: 0,
            position: Self::INITIAL_BUFFER_POSITION,
            overrun: false,
            buffer: vec![0u8; NETWORKMESSAGE_MAXSIZE as usize],
        }
    }

    pub fn reset(&mut self) {
        self.length = 0;
        self.position = Self::INITIAL_BUFFER_POSITION;
        self.overrun = false;
    }

    // === Getters liknande C++ ===
    pub fn get_length(&self) -> u16 {
        self.length
    }

    pub fn get_buffer_position(&self) -> u16 {
        self.position
    }

    pub fn set_buffer_position(&mut self, pos: u16) -> bool {
        let max = (self.buffer.len() as u16).saturating_sub(Self::INITIAL_BUFFER_POSITION);
        if pos < max {
            self.position = pos + Self::INITIAL_BUFFER_POSITION;
            true
        } else {
            false
        }
    }

    /// Läs en byte
    pub fn get_byte(&mut self) -> u8 {
        if !self.can_read(1) {
            return 0;
        }
        let b = self.buffer[self.position as usize];
        self.position += 1;
        b
    }

    pub fn get_previous_byte(&mut self) -> u8 {
        self.position -= 1;
        self.buffer[self.position as usize]
    }

    pub fn get<T: Copy + Default>(&mut self) -> T {
        let size = std::mem::size_of::<T>();
        if !self.can_read(size as i32) {
            return T::default();
        }

        let mut tmp = vec![0u8; size];
        tmp.copy_from_slice(&self.buffer[self.position as usize..self.position as usize + size]);
        self.position += size as u16;

        unsafe { std::ptr::read_unaligned(tmp.as_ptr() as *const T) }
    }

    pub fn get_string(&mut self, string_len: Option<u16>) -> String {
        let len = string_len.unwrap_or_else(|| self.get_u16());
        if !self.can_read(len as i32) {
            return String::new();
        }

        let start = self.position as usize;
        let end = start + len as usize;
        self.position += len;

        String::from_utf8_lossy(&self.buffer[start..end]).into_owned()
    }

    // set

    pub fn set_length(&mut self, new_len: u16) {
        self.length = new_len;
    }

    pub fn skip_bytes(&mut self, count: i16) {
        self.position = (self.position as i16 + count) as u16;
    }

    pub fn add_byte(&mut self, value: u8) {
        if !self.can_add(1) {
            return;
        }
        self.buffer[self.position as usize] = value;
        self.position += 1;
        self.length += 1;
    }

    pub fn add<T: Copy>(&mut self, value: T) {
        let size = std::mem::size_of::<T>();
        if !self.can_add(size) {
            return;
        }
        let ptr = &value as *const T as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
        self.buffer[self.position as usize..self.position as usize + size].copy_from_slice(slice);
        self.position += size as u16;
        self.length += size as u16;
    }

    pub fn add_string(&mut self, value: &str) {
        let string_len = value.len();
        if !self.can_add(string_len + 2) || string_len > 8192 {
            return;
        }
        self.add::<u16>(string_len as u16);
        self.buffer[self.position as usize..self.position as usize + string_len]
            .copy_from_slice(value.as_bytes());
        self.position += string_len as u16;
        self.length += string_len as u16;
    }

    // === Helpers ===

    fn can_add(&self, size: usize) -> bool {
        (size + self.position as usize) < Self::MAX_BODY_LENGTH
    }

    fn can_read(&mut self, size: i32) -> bool {
        if (self.position as i32 + size) > (self.length as i32 + 8)
            || size >= (NETWORKMESSAGE_MAXSIZE - self.position as i32)
        {
            self.overrun = true;
            return false;
        }
        true
    }

    pub fn get_u16(&mut self) -> u16 {
        if !self.can_read(2) {
            return 0;
        }
        let pos = self.position as usize;
        let val = u16::from_le_bytes([self.buffer[pos], self.buffer[pos + 1]]);
        self.position += 2;
        val
    }

    pub fn get_u32(&mut self) -> u32 {
        if !self.can_read(4) {
            return 0;
        }
        let pos = self.position as usize;
        let val = u32::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
        ]);
        self.position += 4;
        val
    }

    pub fn get_u64(&mut self) -> u64 {
        if !self.can_read(8) {
            return 0;
        }
        let pos = self.position as usize;
        let val = u64::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
            self.buffer[pos + 4],
            self.buffer[pos + 5],
            self.buffer[pos + 6],
            self.buffer[pos + 7],
        ]);
        self.position += 8;
        val
    }

    /// Läs ut `n` råbytes från bufferten och flytta positionen
    pub fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        let start = self.position as usize;
        let end = (start + n).min(self.buffer.len());
        let out = self.buffer[start..end].to_vec();
        self.position = end as u16;
        out
    }

}



