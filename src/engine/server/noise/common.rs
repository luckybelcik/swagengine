use glam::IVec2;

use crate::engine::server::constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE};

#[derive(Clone, Copy)]
pub struct NoiseLayer2D {
    pub layer: [u8; CHUNK_BLOCK_COUNT as usize],
}

impl NoiseLayer2D {
    pub fn new() -> NoiseLayer2D {
        NoiseLayer2D {
            layer: [0; CHUNK_BLOCK_COUNT as usize],
        }
    }

    pub fn write(&mut self, chunk_relative_pos: IVec2, value: u8) {
        self.layer[chunk_relative_pos.y as usize * CHUNK_SIZE as usize + chunk_relative_pos.x as usize] = value;
    }

    pub fn read(&self, chunk_relative_pos: IVec2) -> u8 {
        return self.layer[chunk_relative_pos.y as usize * CHUNK_SIZE as usize + chunk_relative_pos.x as usize];
    }

    pub fn read_index(&self, index: usize) -> u8 {
        return self.layer[index];
    }
}

#[derive(Clone, Copy)]
pub struct NoiseLayer1D {
    layer: [f32; CHUNK_SIZE as usize],
}

impl NoiseLayer1D {
    pub fn new() -> NoiseLayer1D {
        NoiseLayer1D {
            layer: [0.0; CHUNK_SIZE as usize]
        }
    }

    pub fn write(&mut self, x: i32, value: f32) {
        self.layer[x as usize] = value;
    }

    pub fn read(&self, x: i32) -> f32 {
        return self.layer[x as usize];
    }
}