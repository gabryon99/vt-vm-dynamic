pub trait Addressable<T> {
    fn read(&self, address: usize) -> T;
    fn write(&mut self, address: usize, value: T);
    fn write_chunk(&mut self, chunk: Vec<T>) -> Result<(), String>;
}

pub const MEMORY_SIZE: usize = 1024 * 64;

pub struct Memory {
    data: [u8; MEMORY_SIZE], // Reserve 64KB for programs
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            data: [0; MEMORY_SIZE],
        }
    }
}

impl Addressable<u8> for Memory {
    fn read(&self, address: usize) -> u8 {
        self.data[address]
    }

    fn write(&mut self, address: usize, value: u8) {
        self.data[address] = value;
    }

    fn write_chunk(&mut self, chunk: Vec<u8>) -> Result<(), String> {
        if chunk.len() > MEMORY_SIZE {
            return Err(format!(
                "Chunk size is larger than maximum memory ({} bytes)",
                MEMORY_SIZE
            ));
        }

        for (index, data) in chunk.iter().enumerate() {
            self.data[index] = *data;
        }

        Ok(())
    }
}
