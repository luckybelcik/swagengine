use crate::engine::{command_registry::{self, DebugCommandWithArgs}, state::State, time::Time};
use winit::{application::ApplicationHandler, event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use std::{sync::{mpsc::Receiver, Arc}};
use crate::engine::util::{ClientConfig};

pub struct Client {
    state: Option<State>,
    pub time: Time,
    console_listener: Receiver<DebugCommandWithArgs>,
    server_listener: Receiver<String>,
    pub client_config: ClientConfig,
}

impl Client {
    pub fn new(console_listener: Receiver<DebugCommandWithArgs>, server_listener: Receiver<String>) -> Self {
        Self {
            state: None,
            time: Time::new(),
            console_listener: console_listener,
            server_listener: server_listener,
            client_config: ClientConfig::default(),
        }
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
}