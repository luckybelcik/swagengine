pub struct ChunkRelativePos {
    pub x: usize,
    pub y: usize,
}

impl ChunkRelativePos {
    pub fn new(x: usize, y: usize) -> ChunkRelativePos {
        return ChunkRelativePos {
            x,
            y,
        }
    }
}