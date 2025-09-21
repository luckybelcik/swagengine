use std::sync::Arc;
use wgpu::util::DeviceExt;

use winit::
    window::Window
;

use crate::engine::{client::{client::Client, client_chunk::ClientChunk}, server::constants::CHUNK_BLOCK_COUNT};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor{
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 20,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::Immediate,
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                ],
                push_constant_ranges: &[
                    wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX,
                        range: 0..20,
                    }
                ],
            });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ClientChunk::get_desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
                depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        surface.configure(&device, &config);

        let state = State {
            window,
            device,
            queue,
            config,
            render_pipeline,
            size,
            surface,
            surface_format,
        };

        state
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&self, client: &Client) {
        self.window.request_redraw();

        let surface_texture = self.surface.get_current_texture().expect("failed to acquire next swapchain texture");

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            let window_size = self.get_window().inner_size();
            let window_size = [window_size.width as f32, window_size.height as f32];
            // Push window size
            render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            8,
            bytemuck::bytes_of(&window_size));

            // Push zoom factor
            render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            16,
            bytemuck::bytes_of(&client.get_zoom_factor()));
            for chunk in client.get_chunks() {
                chunk.prepare_for_draw(&mut render_pass);
                render_pass.draw(0..6, 0..(CHUNK_BLOCK_COUNT as u32 * 3));
            }
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}