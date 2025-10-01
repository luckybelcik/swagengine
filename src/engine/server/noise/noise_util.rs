use glam::IVec2;

#[inline]
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

const IDW_POWER: f32 = 2.0;

pub fn interpolate_idw(
    block_pos_local: IVec2, 
    sampled_points: &[(IVec2, f32)],
) -> f32 {
    let mut total_weight: f32 = 0.0;
    let mut weighted_sum: f32 = 0.0;

    const EPSILON: f32 = 0.0001; 

    for (point_pos, point_value) in sampled_points.iter() {
        let dx = block_pos_local.x as f32 - point_pos.x as f32;
        let dy = block_pos_local.y as f32 - point_pos.y as f32;
        
        let distance_sq = dx * dx + dy * dy;
        
        if distance_sq < EPSILON {
            return *point_value;
        }
        
        let distance = distance_sq.sqrt();
        
        let weight = 1.0 / distance.powf(IDW_POWER); 
        
        weighted_sum += weight * point_value;
        total_weight += weight;
    }

    if total_weight > EPSILON {
        weighted_sum / total_weight
    } else {
        sampled_points.first().map(|(_, v)| *v).unwrap_or(0.0) 
    }
}

pub fn get_chunk_seed(world_seed: i32, chunk_pos: &IVec2) -> u64 {
    let s = world_seed as u32 as u64;
    let xx = chunk_pos.x as u32 as u64;
    let yy = chunk_pos.y as u32 as u64;
    
    let key = s.wrapping_mul(0xC2B2AE3D27D4EB4F)
                ^ xx.wrapping_mul(0x165667B19E3779F9)
                ^ yy.wrapping_mul(0x9E3779B97F4A7C15);
    splitmix64(key)
}