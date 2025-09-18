use glam::IVec2;

use crate::engine::common::{Block, ChunkMesh};
use wgpu::util::DeviceExt;

pub struct ClientChunk {
    buffer: wgpu::Buffer,
    mesh: ChunkMesh
}

impl ClientChunk {
    pub fn create(position: IVec2, mesh: ChunkMesh, device: &wgpu::Device) -> ClientChunk {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}x {}y Chunk Buffer", position.x, position.y)),
                contents: bytemuck::bytes_of(&mesh),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        ClientChunk {
            buffer: vertex_buffer,
            mesh,
        }
    }

    pub fn get_desc() -> wgpu::VertexBufferLayout<'static> {
        // 0 = position, 1 = blockid, 2 = blocktype, 3 = textureindex
        const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0 => Uint8x2, 1 => Uint16, 2 => Uint8, 3 => Uint8];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Block>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}