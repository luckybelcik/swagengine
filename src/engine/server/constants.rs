// stay within a size of 32 because otherwise the stack might overflow
// thats because while constructed, the chunk is stored on the stack
// it is only put on the heap once it enters the hashmap
// previously the limit was 64 but decoding packets made the stack overflow
// oops?
pub const CHUNK_SIZE: u8 = 32;
pub const CHUNK_BLOCK_COUNT: u16 = CHUNK_SIZE as u16 * CHUNK_SIZE as u16;
pub const BIOME_SAMPLE_POINT_AMOUNT: usize = CHUNK_SIZE as usize / 8;
pub const BIOME_MAP_GRID_SIZE: usize = 100;

pub const CONTINENTAL_NINDEX: usize = 0;
pub const MOUNTAINOUS_NINDEX: usize = 1;
pub const HILLY_NINDEX: usize = 2;
pub const TEXTURE_NINDEX: usize = 3;
pub const CELLULAR_NINDEX: usize = 4;
pub const GRIDLIKE_NINDEX: usize = 5;
pub const NUM_1D_NOISE_LAYERS: usize = 6;

pub const TEMPERATURE_INDEX: usize = 0;
pub const HUMIDITY_INDEX: usize = 1;
pub const NUM_2D_NOISE_LAYERS: usize = 2;

pub const TICK_RATE: u64 = 60;