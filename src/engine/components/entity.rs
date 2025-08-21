use bitflags::bitflags;

pub struct EntityID {
    id: u32
}

pub enum Components {
    basic,
    basic_bitflags,
    mass,
}

pub struct BasicComponent {
    maxHealth: u32,
    currentHealth: u32,
    position: Vec2,
    velocity: Vec2,
    size: Vec2
}

bitflags!{
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BasicBitflags : u8 {
        const INVULNERABLE = 0b00000001;
    }
}

pub impl BasicBitflags {
    //getters
    pub fn is_invulnerable(&self) -> bool {
        self.contains(BasicBitflags::INVULNERABLE)
    }

    //setters
    pub fn set_invulnerable(&mut self, state: bool) {
        self.set(BasicBitflags::INVULNERABLE, state);
    } 
}

pub struct MassComponent {
    mass: float,
}