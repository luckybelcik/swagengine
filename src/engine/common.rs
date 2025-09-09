#[derive(Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        return Vec2 {
            x,
            y,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> IVec2 {
        return IVec2 {
            x,
            y,
        }
    }
}

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