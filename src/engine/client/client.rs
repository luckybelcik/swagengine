use crate::engine::{client::state::State, command_registry::{self, DebugCommandWithArgs}, common::{ChunkMesh, ServerPacket}, time::Time};
use glam::IVec2;
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use std::{collections::HashMap, sync::{mpsc::Receiver, Arc}};

pub struct Client {
    state: Option<State>,
    pub time: Time,
    console_listener: Receiver<DebugCommandWithArgs>,
    server_listener: Receiver<Vec<u8>>,
    pub client_config: ClientConfig,
    player_uuid: u64,
    player_nickname: String,
    loaded_chunks: HashMap<IVec2, ChunkMesh>,
}

impl Client {
    pub fn new(console_listener: Receiver<DebugCommandWithArgs>, server_listener: Receiver<Vec<u8>>) -> Self {
        Self {
            state: None,
            time: Time::new(),
            console_listener: console_listener,
            server_listener: server_listener,
            client_config: ClientConfig::default(),
            player_uuid: fastrand::u64(..),
            player_nickname: "playerboy".to_string(),
            loaded_chunks: HashMap::new(),
        }
    }

    fn redraw(&mut self) {
        self.time.update();
        self.on_handle_command();
        self.on_handle_server_packet();
        self.on_update_frame();
        self.on_render();
    }

    fn resize(&mut self, size: &PhysicalSize<u32> ) {
        if let Some(state) = &mut self.state {
            state.resize(*size);
        }
    }

    fn close(&self, event_loop: &ActiveEventLoop) {
        event_loop.exit();
    }
}

impl ApplicationHandler for Client {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
                .with_title("swagrarria")
                .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        window.request_redraw();
        Client::on_launch(self);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(_state) = &mut self.state {
            match &event {
                WindowEvent::CloseRequested => self.close(event_loop),
                WindowEvent::RedrawRequested => self.redraw(),
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key_code),
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    self.on_key_pressed(*key_code);
                }

                _ => {
                    return;
                }
            }
        }
    }
}

impl Client {
    fn on_launch(&mut self) {

    }

    fn on_update_frame(&mut self) {
        // Input/UI/scripting here
    }

    fn on_render(&mut self) {
        if let Some(state) = &mut self.state {
            state.render();
        }
    }

    fn on_key_pressed(&mut self, key: KeyCode) {
        match key {
        KeyCode::KeyP => {
            let fps = self.time.average_fps();
            println!("Average FPS: {:.2}", fps);
        }
        KeyCode::KeyO => {
            self.time.reset_average_fps();
        }
        _ => {}
        }
    }

    fn on_handle_command(&mut self) {
        while let Ok(cmd) = self.console_listener.try_recv() {
            command_registry::handle_client_command(self, &cmd);
        }
    }

    fn on_handle_server_packet(&mut self) {
        while let Ok(raw_packet) = self.server_listener.try_recv() {
            let (packet, bytes_consumed) = bincode::decode_from_slice(&raw_packet, bincode::config::standard()).unwrap();
            println!("v Bytes read: {} bytes", bytes_consumed);
            match packet {
                ServerPacket::ChunkMesh(mesh) => {
                    let coord = IVec2::new(mesh.0.0, mesh.0.1);
                    if self.loaded_chunks.contains_key(&coord) {
                        println!("Got mesh already at {}x {}y!", mesh.0.0, mesh.0.1);
                    } else {
                        println!("Got mesh at position {}x {}y!", mesh.0.0, mesh.0.1);
                        self.loaded_chunks.insert(coord, *mesh.1);
                    }
                },
                ServerPacket::BlockChange(block_change) => {
                    println!("Got block change at {}x {}y in layer {:?} with blocktype {:?} and block_id {}", block_change.0.0, block_change.0.1, block_change.1.layer, block_change.1.block_type, block_change.1.block_id)
                },
                ServerPacket::Message(message) => {
                    println!("Got message {}!", message)
                },
                ServerPacket::Ping => {
                    println!("Got pinged!")
                }
            }
        }
    }
    
    pub fn get_uuid(&self) -> u64 {
        return self.player_uuid;
    }

    pub fn get_uuid_string(&self) -> String {
        let hex_string = format!("{:016x}", self.player_uuid);

        let dashed_hex: String = hex_string
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 4 == 0 {
                    vec!['-', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        return dashed_hex;
    }

    pub fn get_nickname(&self) -> &String {
        return &self.player_nickname;
    }
}

pub struct ClientConfig {
    pub frame_cap: u32,
    pub vsync: bool
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self { 
            frame_cap: 60, 
            vsync: true 
        }
    }
}