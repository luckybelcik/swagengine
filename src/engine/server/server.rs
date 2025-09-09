use std::collections::{hash_map::Keys, HashMap};
use crate::engine::server::world::Dimension;

pub struct Server {
    pub dimensions: HashMap<String, Dimension>
}

impl Server {
    pub fn start_server() -> Server {
        let mut starting_dimensions: HashMap<String, Dimension> = HashMap::new();
        let basic_dimension: Dimension = Dimension::new_basic_dimension();
        starting_dimensions.insert(basic_dimension.name.clone(), basic_dimension);
        return Server { dimensions: (starting_dimensions) }
    }

    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        return self.dimensions.get(name);
    }

    pub fn get_dimension_keys(&self) -> Keys<'_, String, Dimension> {
        return self.dimensions.keys();
    }
}