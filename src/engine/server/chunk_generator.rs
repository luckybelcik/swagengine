use std::{collections::HashSet, sync::mpsc::{Receiver, Sender}};

use glam::IVec2;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::engine::server::{biome::{BiomeMap, BiomeRegistry}, chunk::Chunk};

pub struct ChunkGenerator {
    chunks_awaiting_generation: HashSet<IVec2>
}

impl ChunkGenerator {
    pub fn new(server_listener: Receiver<IVec2>, biome_registry: BiomeRegistry) {
        let (generator_sender, generator_listener) = std::sync::mpsc::channel::<Chunk>();

        std::thread::spawn(move || {
            println!("Chunk generator thread spawned");
            const PARALLEL_THRESHOLD: usize = 4;
            let biome_map = biome_registry.biome_map;

            loop {
                let mut batch: Vec<IVec2> = match server_listener.recv() {
                    Ok(coords) => vec![coords],
                    Err(_) => {
                        println!("Server listener disconnected. Generator shutting down.");
                        break;
                    }
                };

                while let Ok(coords) = server_listener.try_recv() {
                    batch.push(coords);
                }

                let batch_size = batch.len();

                let generated_chunks: Vec<Chunk> = if batch_size >= PARALLEL_THRESHOLD {
                    println!("Processing large batch of {} chunks in parallel.", batch_size);
                    batch
                        .par_iter()
                        .map(|&coords| Chunk::generate_chunk(&coords, &biome_map))
                        .collect()
                } else {
                    println!("Processing small batch of {} chunks sequentially.", batch_size);
                    batch
                        .into_iter()
                        .map(|coords| Chunk::generate_chunk(&coords, &biome_map))
                        .collect()
                };
            }
        });
    }

    pub fn load_chunk(&mut self, chunk_pos: &IVec2) {
        self.chunks_awaiting_generation.insert(chunk_pos.clone());
    }
}