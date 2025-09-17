// stay within a size of 64 because otherwise the stack might overflow
// thats because while constructed, the chunk is stored on the stack
// it is only put on the heap once it enters the hashmap
pub const CHUNK_SIZE: u8 = 64;
pub const CHUNK_BLOCK_COUNT: u16 = CHUNK_SIZE as u16 * CHUNK_SIZE as u16;

pub const TICK_RATE: u64 = 60;