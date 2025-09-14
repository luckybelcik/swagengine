use std::{collections::{hash_map::Keys, HashMap}, sync::mpsc::Receiver};
use crate::engine::{command_registry::{self, DebugCommandWithArgs}, server::world::Dimension};

pub struct Server {
    pub dimensions: HashMap<String, Dimension>,
    running: bool,
    console_listener: Receiver<DebugCommandWithArgs>,
}

impl Server {
    pub fn start_server(console_listener: Receiver<DebugCommandWithArgs>) -> Server {
        let mut starting_dimensions: HashMap<String, Dimension> = HashMap::new();
        let basic_dimension: Dimension = Dimension::new_basic_dimension();
        starting_dimensions.insert(basic_dimension.name.clone(), basic_dimension);
        return Server {
            dimensions: starting_dimensions,
            running: true,
            console_listener: console_listener,
        }
    }

    pub fn stop_server(&mut self) {
        println!("Stopping server!");
        // TODO: Nothing here yet, add saving later
        self.running = false;
    }

    pub fn on_tick(&mut self) {
        for dimension in self.dimensions.values_mut() {
            dimension.load_chunks();
        }
    }

    pub fn process_commands(&mut self) {
        while let Ok(cmd) = self.console_listener.try_recv() {
            command_registry::handle_server_command(self, &cmd);
        }
    }

    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        return self.dimensions.get(name);
    }

    pub fn get_dimension_keys(&self) -> Keys<'_, String, Dimension> {
        return self.dimensions.keys();
    }

    pub fn is_running(&self) -> bool {
        return self.running;
    }
}