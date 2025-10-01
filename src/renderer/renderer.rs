use imgui::MouseCursor;
use specs::prelude::*;

use super::{
    DeltaTimer,
    command_queue::{CommandQueue, RenderMeshCommand},
    composition_pass::CompositionPass,
    deferred_pass::DeferredPass,
    lights::LightsResources,
    meshes::MeshResources,
    scene_base::SceneBaseResources,
};
use crate::renderer::command_queue::RenderBatch;
use crate::renderer::material::MaterialResources;
use crate::renderer::shadow_passes::{RenderShadowBatch, RenderShadowMeshCommand, ShadowPasses};
use crate::renderer::ssao_pass::SSAOPass;
use std::time::Instant;
use wgpu::naga::SwitchValue::Default;

pub enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub last_cursor: Option<MouseCursor>,
    is_surface_ready: bool,
}

impl Renderer {
    pub async fn new(
        window: std::sync::Arc<winit::window::Window>,
    ) -> (Self, wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..wgpu::InstanceDescriptor::default()
        });
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        log::info!("Limits: {:?}", adapter.limits());

        // Todo: Specify required features
        let adapter_features = adapter.features();
        log::info!("Features: {:?}", adapter_features);

        // Todo: Specify limits
        let required_limits = wgpu::Limits {
            max_bind_groups: 6,
            ..wgpu::Limits::default()
        };
        log::info!("Limits: {:#?}", required_limits);

        // todo: Add back tracing
        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::DEPTH_CLIP_CONTROL,
                required_limits,
                //trace: wgpu::Trace::Directory(trace_dir.ok().as_ref().map(std::path::Path::new)),
                trace: wgpu::Trace::Off,
                memory_hints: wgpu::MemoryHints::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        (
            Renderer {
                instance,
                size,
                surface,
                config,
                adapter,
                last_cursor: None,
                is_surface_ready: false,
            },
            device,
            queue,
        )
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&device, &self.config);
            self.is_surface_ready = true;
        }
        // todo: Call resize on passes
    }
}

// Render System

impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, RendererEvent>,
        WriteExpect<'a, DeltaTimer>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
        ReadExpect<'a, DeferredPass>,
        ReadExpect<'a, SceneBaseResources>,
        ReadExpect<'a, MeshResources>,
        ReadExpect<'a, MaterialResources>,
        WriteExpect<'a, CommandQueue<RenderMeshCommand, RenderBatch>>,
        ReadExpect<'a, ShadowPasses>,
        WriteExpect<'a, CommandQueue<RenderShadowMeshCommand, RenderShadowBatch>>,
        ReadExpect<'a, SSAOPass>,
        ReadExpect<'a, CompositionPass>,
        ReadExpect<'a, LightsResources>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut event,
            mut d_t,
            device,
            queue,
            deferred_pass,
            scene_base_resources,
            mesh_resources,
            material_resources,
            mut mesh_commands,
            shadow_passes,
            mut shadow_mesh_commands,
            ssao_pass,
            composition_pass,
            lights_resources,
        ) = data;

        match *event {
            RendererEvent::Render => {
                if self.is_surface_ready {
                    deferred_pass.render(
                        &device,
                        &queue,
                        &scene_base_resources,
                        &mesh_resources,
                        &material_resources,
                        &mut mesh_commands,
                    );
                    ssao_pass.render(&device, &queue, &scene_base_resources, &deferred_pass);
                    shadow_passes.render(&device, &queue, &mesh_resources, &mut shadow_mesh_commands);
                    composition_pass.render(
                        &device,
                        &queue,
                        &mut self.surface,
                        &scene_base_resources,
                        &lights_resources,
                        &deferred_pass,
                        &shadow_passes,
                        &ssao_pass,
                    );

                    *event = RendererEvent::None;
                    *d_t = DeltaTimer::new(Instant::now() - d_t.get_last_render(), Instant::now());
                }
            }
            RendererEvent::Resize(size) => {
                self.resize(size, &device);
                *event = RendererEvent::None;
            }
            _ => (),
        }
    }
}
