use clap::{ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum Environment {
    Both,
    Client,
    Server,
}