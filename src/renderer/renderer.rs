use imgui::MouseCursor;
use specs::prelude::*;

use std::{time::Instant};

use super::{command_queue::{CommandQueue, RenderMeshCommand}, composition_pass::CompositionPass, deferred_pass::DeferredPass, DeltaTimer, lights::{LightsResources}, meshes::MeshResources, scene_base::SceneBaseResources};

pub enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub last_cursor: Option<MouseCursor>,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> (Self, wgpu::Device, wgpu::Queue) {

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        log::info!("Limits: {:?}", adapter.limits());

        // Todo: Specify required features
        let adapter_features = adapter.features();
        log::info!("Features: {:?}", adapter_features);

        // Todo: Specify limits
        let limits = wgpu::Limits {
            max_bind_groups: 6,
            ..wgpu::Limits::default()
        };
        log::info!("Limits: {:#?}", limits);

        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: limits,
                    shader_validation: true,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        ( Renderer {
            swap_chain,
            instance,
            size,
            surface,
            adapter,
            sc_desc,
            last_cursor: None,
        }, device, queue )
    }

    fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device
    ) {

        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = device.create_swap_chain(&self.surface, &self.sc_desc);

        // Call resize on passes
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
        WriteExpect<'a, CommandQueue<RenderMeshCommand>>,
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
            mut mesh_commands,
            composition_pass,
            lights_resources
        ) = data;

        match *event {
            RendererEvent::Render => {

                deferred_pass.render(&device, &queue, &scene_base_resources, &mesh_resources, &mut mesh_commands);
                composition_pass.render(&device, &queue, &mut self.swap_chain, &lights_resources);

                *event = RendererEvent::None;
                *d_t = DeltaTimer::new(Instant::now() - d_t.get_last_render(), Instant::now());
            }
            RendererEvent::Resize(size) => {
               self.resize(size, &device);
               *event = RendererEvent::None;
            }
            _ => (),
        }
    }
}
