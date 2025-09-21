use std::{collections::HashMap, time::{Duration, Instant}};
use dashmap::DashMap;
use glam::{IVec2, UVec2};
use hecs::World;
use noise_functions::{modifiers::Frequency, Noise, OpenSimplex2};

use crate::engine::{components::alive::{AliveTask, AliveTaskKey, EntityID, PlayerID}, server::{chunk::{self, Chunk}, common::BasicNoiseGenerators}};

pub struct Dimension {
    pub name: String,
    pub size: UVec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    noise_generators: BasicNoiseGenerators,
    pub players: HashMap<PlayerID, hecs::Entity>,
    player_tasks: DashMap<AliveTaskKey, AliveTask>,
    entities: HashMap<EntityID, hecs::Entity>,
    entity_tasks: DashMap<AliveTaskKey, AliveTask>,
}

impl Dimension {
    pub fn new_basic_dimension(seed: i32) -> Dimension {
        let noise_generator: Frequency<OpenSimplex2, noise_functions::Constant> = OpenSimplex2::default()
            .frequency(0.025);
        return Dimension { 
            name: "basic_dimension".to_string(),
            size: UVec2 { x: (100), y: (100) },
            ecs_world: World::new(),
            chunks: HashMap::new(),
            noise_generators: BasicNoiseGenerators::new(seed),
            players: HashMap::new(),
            player_tasks: DashMap::new(),
            entities: HashMap::new(),
            entity_tasks: DashMap::new(),
        }
    }

    pub fn load_chunks(&mut self) {
        let generated_height = 8;
        let generated_width = 60;

        let half_height = generated_height / 2;
        let half_width = generated_width / 2;

        for x in -half_width..half_width {
            for y in -half_height..half_height {
                let chunk_pos = IVec2::new(x, y);
                self.try_load_chunk(chunk_pos);
            }
        }
        
    }

    pub fn get_chunks(&self) -> Vec<(&IVec2, &Chunk)> {
        self.chunks.iter().collect::<Vec<(&IVec2, &Chunk)>>()
    }

    fn chunk_at(&self, pos: &IVec2) -> bool {
        return self.chunks.contains_key(&pos);
    }

    fn chunk_within_world_bounds(&self, pos: &IVec2) -> bool {
        let half_x = self.size.x as i32 / 2;
        let half_y = self.size.y as i32 / 2;

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
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos, &self.noise_generators);
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
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos, &self.noise_generators);
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