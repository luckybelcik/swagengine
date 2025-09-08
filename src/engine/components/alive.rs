use crate::engine::common::Vec2;

const IS_PLAYER_BIT: u64 = 1 << 63;

pub struct PlayerID {
    pub id: u32,
}

pub struct EntityID {
    pub id: u32,
}

#[derive(PartialEq, Eq, Hash)]
pub struct AliveTaskKey {
    pub key: u64,
}

impl AliveTaskKey {
    pub fn new_entity_task(entity_id: &EntityID, alive_component: &AliveComponents) -> AliveTaskKey {
        let mut key = (entity_id.id as u64) << 32 | (*alive_component as u64);
        AliveTaskKey { key }
    }

    pub fn new_player_task(player_id: &PlayerID, alive_component: &AliveComponents) -> AliveTaskKey {
        let mut key = (player_id.id as u64) << 32 | (*alive_component as u64);
        key |= IS_PLAYER_BIT;
        AliveTaskKey { key }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AliveComponents {
    Basic,
    Gravity,
    IsEntity,
    IsPlayer,
}

pub trait AliveComponent {}
impl AliveComponent for Basic {}
impl AliveComponent for Gravity {}
impl AliveComponent for IsEntity {}
impl AliveComponent for IsPlayer {}

pub struct AliveTask {
    pub component: AliveComponents,
    pub action: Action,
}

pub enum Action {
    Set(String, u64),
    Add(String, u64),
    Subtract(String, u64),
    Multiply(String, u64),
    Divide(String, u64)
}

pub struct Basic {
    max_health: u32,
    current_health: u32,
    position: Vec2,
    velocity: Vec2,
    size: Vec2
}

pub struct Gravity {
    mass: f32,
}

pub struct IsEntity {
    entity_name: String,
    entity_id: EntityID,
    entity_type: String,
}

pub struct IsPlayer {
    player_name: String,
    player_id: PlayerID,
}