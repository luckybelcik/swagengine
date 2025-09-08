use std::collections::HashMap;
use dashmap::DashMap;
use hecs::World;

use crate::engine::{common::{IVec2, Vec2}, components::alive::{AliveTask, AliveTaskKey, EntityID, PlayerID}, server::chunk::Chunk};

pub struct Dimension {
    pub name: String,
    size: IVec2,
    ecs_world: hecs::World,
    chunks: HashMap<IVec2, Chunk>,
    players: HashMap<PlayerID, hecs::Entity>,
    player_tasks: DashMap<AliveTaskKey, AliveTask>,
    entities: HashMap<EntityID, hecs::Entity>,
    entity_tasks: DashMap<AliveTaskKey, AliveTask>
}

impl Dimension {
    pub fn new_basic_dimension() -> Dimension {
        return Dimension { 
            name: ("basic_dimension".to_string()),
            size: (IVec2 { x: (10), y: (10) }),
            ecs_world: (World::new()),
            chunks: (HashMap::new()),
            players: (HashMap::new()),
            player_tasks: (DashMap::new()),
            entities: (HashMap::new()),
            entity_tasks: (DashMap::new()) }
    }
}