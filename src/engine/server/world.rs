use std::{collections::HashMap, time::{Duration, Instant}};
use dashmap::DashMap;
use hecs::World;

use crate::engine::{common::{IVec2}, components::alive::{AliveTask, AliveTaskKey, EntityID, PlayerID}, server::chunk::Chunk};

pub struct Dimension {
    pub name: String,
    pub size: IVec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    pub players: HashMap<PlayerID, hecs::Entity>,
    player_tasks: DashMap<AliveTaskKey, AliveTask>,
    entities: HashMap<EntityID, hecs::Entity>,
    entity_tasks: DashMap<AliveTaskKey, AliveTask>,
}

impl Dimension {
    pub fn new_basic_dimension() -> Dimension {
        return Dimension { 
            name: ("basic_dimension".to_string()),
            size: (IVec2 { x: (100), y: (100) }),
            ecs_world: (World::new()),
            chunks: (HashMap::new()),
            players: (HashMap::new()),
            player_tasks: (DashMap::new()),
            entities: (HashMap::new()),
            entity_tasks: (DashMap::new()),
        }
    }

    pub fn load_chunks(&mut self) {
        let generated_size = 2;

        let half_size = generated_size / 2;

        for x in -half_size..half_size {
            for y in -half_size..half_size {
                let chunk_pos = IVec2::new(x, y);
                self.try_load_chunk(chunk_pos);
            }
        }
        
    }

    fn chunk_at(&self, pos: &IVec2) -> bool {
        return self.chunks.contains_key(&pos);
    }

    fn chunk_within_world_bounds(&self, pos: &IVec2) -> bool {
        let half_x = self.size.x / 2;
        let half_y = self.size.y / 2;

        let is_x_within_bounds = pos.x >= -half_x && pos.x < half_x;
        let is_y_within_bounds = pos.y >= -half_y && pos.y < half_y;

        is_x_within_bounds && is_y_within_bounds
    }

    fn try_load_chunk(&mut self, chunk_pos: IVec2) {
        if !self.chunk_within_world_bounds(&chunk_pos) {
            println!("Chunk out of bounds {:?}", &chunk_pos)
        } else if self.chunk_at(&chunk_pos) {
            // println!("Chunk already exists at {:?}", &chunk_pos)
        } else {
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos);
            self.chunks.insert(
                chunk_pos,
                chunk
            );
        }
    }

    pub fn chunk_load_speed_test(&self, chunk_limit: u32) -> Duration {
        let start_time = Instant::now();
        let mut chunks_generated = 0;
        let mut chunk_pos = IVec2::new(0, 0);
        let mut chunks = HashMap::new();

        loop {
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos);
            chunks_generated += 1;
            chunks.insert(
                chunk_pos.clone(),
                chunk
            );

            chunk_pos.x += 1;
            if chunk_pos.x >= 100 {
                chunk_pos.x = 0;
                chunk_pos.y += 1;
            }

            if chunks_generated >= chunk_limit {
                break;
            }
        }

        return start_time.elapsed();
    }
}