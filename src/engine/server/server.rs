use std::{collections::{hash_map::Keys, HashMap}, sync::mpsc::{Receiver, Sender}};
use crate::engine::{command_registry::{self, DebugCommandWithArgs}, common::{PacketHeader, ServerPacket}, server::world::Dimension};

pub struct Server {
    pub dimensions: HashMap<String, Dimension>,
    running: bool,
    console_listener: Receiver<DebugCommandWithArgs>,
    client_sender: Sender<Vec<u8>>,
    pub compress_sent_data: bool,
}

impl Server {
    pub fn start_server(console_listener: Receiver<DebugCommandWithArgs>, client_sender: Sender<Vec<u8>>) -> Server {
        let mut starting_dimensions: HashMap<String, Dimension> = HashMap::new();
        let basic_dimension: Dimension = Dimension::new_basic_dimension(fastrand::i32(..));
        starting_dimensions.insert(basic_dimension.name.clone(), basic_dimension);
        return Server {
            dimensions: starting_dimensions,
            running: true,
            console_listener: console_listener,
            client_sender: client_sender,
            compress_sent_data: true,
        }
    }

    pub fn stop(&mut self) {
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

    pub fn send_packet(&self, packet: ServerPacket) {
        // turn into bytes
        let encoded_packet = bincode::encode_to_vec(packet, bincode::config::standard()).unwrap();
        let mut is_compressed = false;
        let raw_len = encoded_packet.len();

        // compress if large enough
        let encoded_packet: Vec<u8> = if raw_len > 100 && self.compress_sent_data {
            let max_compressed_size = lz4_flex::block::get_maximum_output_size(raw_len);
            let mut compressed_buffer = vec![0u8; max_compressed_size];

            let compressed_len = lz4_flex::compress_into(&encoded_packet, &mut compressed_buffer)
            .map_err(|e| format!("LZ4 Compression Error: {}", e)).unwrap();

            compressed_buffer.truncate(compressed_len);

            is_compressed = true;
            compressed_buffer
        } else {
            encoded_packet
        };

        // Pack into header
        let encoded_packet = PacketHeader {
            is_compressed,
            original_size: raw_len,
            data: encoded_packet,
        };

        // Turn into bytes a second time
        let encoded_packet = bincode::encode_to_vec(encoded_packet, bincode::config::standard()).unwrap();

        let _ = self.client_sender.send(encoded_packet);
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