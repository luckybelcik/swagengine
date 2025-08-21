use std::collections::HashMap;
use crate::engine::{common::{EntityID, IVec2, PlayerID, Vec2}, server::chunk::Chunk};

pub struct Dimension {
    name: String,
    size: Vec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    players: HashMap<PlayerID, hecs::Entity>,
    entities: HashMap<EntityID, hecs::Entity>,
}