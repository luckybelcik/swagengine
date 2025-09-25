use std::{collections::HashMap, path::Path, time::{Duration, Instant}};
use dashmap::DashMap;
use glam::{IVec2, UVec2};
use hecs::World;

use crate::engine::{common::get_data_path, components::alive::{AliveTask, AliveTaskKey, EntityID, PlayerID}, server::{biome::{BiomeRegistry}, chunk::Chunk, data::schema_definitions::{BiomeSchema, DimensionSchema}}};

pub struct Dimension {
    pub name: String,
    pub size: UVec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    biome_registry: BiomeRegistry,
    pub players: HashMap<PlayerID, hecs::Entity>,
    player_tasks: DashMap<AliveTaskKey, AliveTask>,
    entities: HashMap<EntityID, hecs::Entity>,
    entity_tasks: DashMap<AliveTaskKey, AliveTask>,
}

impl Dimension {
    pub fn from_schema(schema: &DimensionSchema, seed: i32) -> Dimension {
        let biomes_result = Self::load_biomes(&schema.name, &get_data_path());

        if let Err(error) = biomes_result {
            println!("No biomes found for dimension {}", &schema.name);
            panic!("Error: {}", error);
        }

        let biome_schemas = biomes_result.unwrap();

        Dimension { 
            name: schema.name.clone(),
            size: schema.size,
            ecs_world: World::new(),
            chunks: HashMap::new(),
            biome_registry: BiomeRegistry::new(biome_schemas, seed),
            players: HashMap::new(),
            player_tasks: DashMap::new(),
            entities: HashMap::new(),
            entity_tasks: DashMap::new(),
        }
    }

    pub fn load_chunks(&mut self) {
        let generated_height = 6;
        let generated_width = 12;

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
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos, &self.biome_registry.biome_map);
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
            let chunk: Chunk = Chunk::generate_chunk(&chunk_pos, &self.biome_registry.biome_map);
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

    pub fn load_dimensions(data_dir: &Path) -> Result<Vec<DimensionSchema>, Box<dyn std::error::Error>> {
        let mut dimensions = Vec::new();
        
        let dimensions_path = data_dir.join("dimensions");

        for entry in std::fs::read_dir(dimensions_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dimension_file_path = path.join("dimension.json");

                let file = std::fs::File::open(dimension_file_path)?;

                let dimension: DimensionSchema = serde_json::from_reader(file)?;

                dimensions.push(dimension);
            }
        }

        Ok(dimensions)
    }

    fn load_biomes(dimension_name: &str, data_dir: &Path) -> Result<Vec<BiomeSchema>, Box<dyn std::error::Error>> {
        let mut biomes = Vec::new();
        
        let biomes_path = data_dir.join("dimensions").join(dimension_name).join("biomes");

        for entry in std::fs::read_dir(biomes_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file = std::fs::File::open(path)?;

                let biome: BiomeSchema = serde_json::from_reader(file)?;

                biomes.push(biome);
            }
        }

        Ok(biomes)
    }
}