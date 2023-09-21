use alloc::vec::Vec;

pub trait SerialBuffer {
    fn new(buffer_size: usize) -> Self where Self: Sized;
    fn write(&mut self, value: u8) -> ();
    fn read(&mut self) -> Option<u8>;
    fn read_all(&mut self) -> Vec<u8>;
    fn reset(&mut self) -> ();
    fn available_to_read(&self) -> usize;
}