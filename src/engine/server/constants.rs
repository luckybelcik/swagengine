// stay within a size of 64 because otherwise the stack might overflow
// thats because while constructed, the chunk is stored on the stack
// it is only put on the heap once it enters the hashmap
pub const CHUNK_SIZE: usize = 63;
pub const CHUNK_BLOCK_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE;