use std::collections::HashMap;
use dashmap::DashMap;

use crate::engine::{common::{IVec2, Vec2}, components::alive::{AliveTask, AliveTaskKey, EntityID, PlayerID}, server::chunk::Chunk};

pub struct Dimension {
    name: String,
    size: Vec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    players: HashMap<PlayerID, hecs::Entity>,
    player_tasks: DashMap<AliveTaskKey, AliveTask>,
    entities: HashMap<EntityID, hecs::Entity>,
    entity_tasks: DashMap<AliveTaskKey, AliveTask>
}

impl Dimension {
}