use std::collections::HashMap;
use crate::engine::server::world::Dimension;

pub struct Server {
    pub worlds: HashMap<&'static str, Dimension>
}

impl Server {

}