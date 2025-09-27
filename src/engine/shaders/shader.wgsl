struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) block_id: u32,
    @location(1) @interpolate(flat) block_type: u32,
    @location(2) @interpolate(flat) texture_index: u32,
};

struct PushConstants {
    chunk_pos: vec2<i32>,
    window_size: vec2<f32>,
    zoom_factor: f32,
};

var<push_constant> pc: PushConstants;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_id: u32,
    @builtin(instance_index) instance_id: u32,
    @location(0) block_id: u32,
    @location(1) position: vec2<u32>,
    @location(2) block_type: u32,
    @location(3) texture_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    var local_pos: vec2<f32>;

    switch vertex_id {
        case 0u: { local_pos = vec2<f32>(0.0, 1.0); }
        case 1u: { local_pos = vec2<f32>(0.0, 0.0); }
        case 2u: { local_pos = vec2<f32>(1.0, 1.0); }
        
        case 3u: { local_pos = vec2<f32>(1.0, 1.0); }
        case 4u: { local_pos = vec2<f32>(0.0, 0.0); }
        case 5u: { local_pos = vec2<f32>(1.0, 0.0); }
        
        default: { /* Should not happen, but required for switch completeness */ }
    }

    var world_pos: vec2<f32>;
    let tile_pos = vec2<f32>(f32(position.x), f32(position.y));
    let chunk_offset = vec2<f32>(f32(pc.chunk_pos.x), f32(pc.chunk_pos.y));

    // Apply tile position and chunk offset
    world_pos = local_pos + tile_pos + chunk_offset;

    let TILE_PIXEL_SIZE: f32 = 16.0;
    
    // Apply scaling
    var final_pos: vec2<f32> = world_pos * TILE_PIXEL_SIZE / pc.window_size;

    // Apply zoom
    final_pos = final_pos * pc.zoom_factor;

    out.clip_position = vec4<f32>(final_pos.x, final_pos.y, 0.0, 1.0);
    out.block_id = block_id;
    out.block_type = block_type;
    out.texture_index = texture_index;
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    if (in.block_type == 0u) {
        discard;
    }

    var color: vec3<f32>;

    switch in.block_id {
        case 0u: { color = vec3(0.45, 0.44, 0.42); } // Stone
        case 1u: { color = vec3(0.32, 0.24, 0.13); } // Dirt
        case 2u: { color = vec3(0.30, 0.62, 0.13); } // Grass
        case 3u: { color = vec3(0.30, 0.42, 0.75); } // Water
        case 4u: { color = vec3(0.89, 0.82, 0.34); } // Sand
        case 5u: { color = vec3(0.88, 0.91, 0.94); } // Snow
        default: { color = vec3(0.00, 0.00, 0.00); } // None
    }

    return vec4<f32>(color, 1.0);
}