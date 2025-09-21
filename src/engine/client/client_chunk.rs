use glam::IVec2;

use crate::engine::{client::state::State, common::{Block, ChunkMesh}, server::constants::CHUNK_SIZE};
use wgpu::{util::DeviceExt, RenderPass};

pub struct ClientChunk {
    position: IVec2,
    buffer: wgpu::Buffer,
    mesh: ChunkMesh
}

impl ClientChunk {
    pub fn create(position: IVec2, mesh: ChunkMesh, device: &wgpu::Device) -> ClientChunk {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}x {}y Chunk Buffer", position.x, position.y)),
                contents: bytemuck::bytes_of(&mesh),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        ClientChunk {
            position,
            buffer,
            mesh,
        }
    }

    pub fn prepare_for_draw(&self, state: &State, render_pass: &mut RenderPass) {
        let pos: [i32; 2] = [self.position.x * CHUNK_SIZE as i32, self.position.y * CHUNK_SIZE as i32];
        state.get_queue().write_buffer(state.get_chunk_offset_buffer(), 0, bytemuck::bytes_of(&pos));
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
    }

    pub fn get_desc() -> wgpu::VertexBufferLayout<'static> {
        // 0 = position, 1 = blockid, 2 = blocktype, 3 = textureindex
        const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0 => Uint32, 1 => Uint8x2, 2 => Uint8, 3 => Uint8];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Block>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}