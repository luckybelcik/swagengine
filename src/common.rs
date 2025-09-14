use std::collections::HashMap;

use clap::{ValueEnum};

use crate::engine::command_registry::DebugCommand;

#[derive(Clone, Debug, ValueEnum)]
pub enum Environment {
    Both,
    Client,
    Server,
}

pub struct CommandRegistry {
    pub global_registry: HashMap<&'static str, DebugCommand>,
}