use crate::engine::{command_registry::{self, DebugCommand}, server::server::Server, state::State, time::Time};
use winit::{application::ApplicationHandler, event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use std::{collections::HashMap, sync::{mpsc::Receiver, Arc}};
use crate::engine::util::{AppConfig};

pub struct App {
    state: Option<State>,
    pub time: Time,
    console_listener: Receiver<String>,
    pub app_config: AppConfig,
    pub command_registry: HashMap<&'static str, DebugCommand>,
    pub server: Option<Server>,
}

impl App {
    pub fn new(listener: Receiver<String>) -> Self {
        Self {
            state: None,
            time: Time::new(),
            console_listener: listener,
            app_config: AppConfig::default(),
            command_registry: command_registry::build_registry(),
            server: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
                .with_title("swagrarria")
                .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        window.request_redraw();
        App::on_launch(self);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let redraw_requested;
        let resized_size;
        if let Some(_state) = &mut self.state {
            match &event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                    return;
                }
                WindowEvent::RedrawRequested => {
                    redraw_requested = true;
                    resized_size = None;
                }
                WindowEvent::Resized(size) => {
                    redraw_requested = false;
                    resized_size = Some(*size);
                }
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
                    redraw_requested = false;
                    resized_size = None;
                }

                _ => {
                    return;
                }
            }
        } else {
            return;
        }

        if redraw_requested {
            self.time.update();
            self.on_handle_command();
            match &mut self.server {
                Some(server) => {
                    App::on_tick(self.time.delta_time(), server);
                },
                None => {},
            }
            self.on_update_frame();
            self.on_render();
            if let Some(state) = &mut self.state {
                state.get_window().request_redraw();
            }
        } else if let Some(size) = resized_size {
            if let Some(state) = &mut self.state {
                state.resize(size);
            }
        }
    }
}

impl App {
    fn on_launch(&mut self) {
        self.server = Some(Server::start_server());
    }
    
    fn on_tick(_delta_time: f32, server: &mut Server) {
        for dimension in server.dimensions.values_mut() {
            dimension.load_chunks();
        }
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
            command_registry::handle_command(self, &cmd);
        }
    }
}