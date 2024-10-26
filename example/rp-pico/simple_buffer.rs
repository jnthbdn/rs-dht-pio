use crate::serial_buffer::SerialBuffer;
use alloc::{collections::VecDeque, vec::Vec};

pub struct SimpleBuffer {
    buf: VecDeque<u8>,
}

#[allow(unused)]
impl SerialBuffer for SimpleBuffer {
    fn new(buffer_size: usize) -> Self {
        assert!(buffer_size > 0);

        Self {
            buf: VecDeque::with_capacity(buffer_size),
        }
    }

    fn write(&mut self, value: u8) {
        if self.buf.len() < self.buf.capacity() {
            self.buf.push_back(value);
        }
    }

    fn read(&mut self) -> Option<u8> {
        self.buf.pop_front()
    }

    fn read_all(&mut self) -> Vec<u8> {
        let mut data = Vec::<u8>::with_capacity(self.available_to_read());

        for _ in 0..self.buf.len() {
            data.push(self.buf.pop_front().unwrap_or(0));
        }

        data
    }

    fn reset(&mut self) {
        self.buf.clear();
    }

    fn available_to_read(&self) -> usize {
        self.buf.len()
    }
}
