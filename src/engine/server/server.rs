use std::collections::HashMap;
use crate::engine::server::world::Dimension;

pub struct Server {
    pub dimensions: HashMap<String, Dimension>
}

impl Server {
    pub fn start_server() -> Server {
        let mut starting_dimensions: HashMap<String, Dimension> = HashMap::new();
        let mut basic_dimension: Dimension = Dimension::new_basic_dimension();
        starting_dimensions.insert(basic_dimension.name.clone(), basic_dimension);
        return Server { dimensions: (starting_dimensions) }
    }
}