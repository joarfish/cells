pub mod renderer;
pub mod scene_base;
pub mod geometry;
pub mod lights;
pub mod meshes;
pub mod deferred_pass;
pub mod shadow_passes;
pub mod utils;
pub mod composition_pass;
pub mod command_queue;
pub mod ssao_pass;
pub mod material;

use std::time::{Duration, Instant};
use specs::prelude::*;

use self::{command_queue::{CommandQueue, RenderMeshCommand}, composition_pass::CompositionPass, deferred_pass::DeferredPass, lights::LightsResources, meshes::MeshResources, renderer::{Renderer, RendererEvent}, scene_base::SceneBaseResources};
use crate::renderer::shadow_passes::{ShadowPasses, RenderShadowMeshCommand, RenderShadowBatch};
use crate::renderer::ssao_pass::SSAOPass;
use crate::renderer::material::MaterialResources;
use crate::renderer::command_queue::RenderBatch;

pub struct DeltaTimer {
    d: Duration,
    last_render: Instant,
}

impl DeltaTimer {

    pub fn new(d : Duration, last_render : Instant) -> Self {
        DeltaTimer {
            d,
            last_render
        }
    }

    pub fn get_duration_f32(&self) -> f32 {
        self.d.as_secs_f32()
    }

    pub fn get_last_render(&self) -> Instant {
        self.last_render
    }
}

pub fn setup_rendering(world: &mut World, window: &winit::window::Window) -> Renderer {

    let window_size = window.inner_size();

    let (renderer, device, queue) = futures::executor::block_on(Renderer::new(&window));

    let mesh_resources = MeshResources::new();
    let lights_resources = LightsResources::new(&device);
    let scene_base_resources = SceneBaseResources::new(&device);
    let material_resources = MaterialResources::new(&device, 20);

    let deferred_pass = DeferredPass::new(&device, &material_resources, &scene_base_resources, window_size.width, window_size.height);
    let shadow_passes = ShadowPasses::new(&device, &mesh_resources, window_size.width, window_size.height);
    let ssao_pass = SSAOPass::new(&device, &queue, &deferred_pass, &scene_base_resources, window_size.width, window_size.height);
    let composition_pass = CompositionPass::new(&device, &queue, &deferred_pass, &shadow_passes, &ssao_pass, &lights_resources, &scene_base_resources);

    world.insert(device);
    world.insert(queue);

    world.insert(mesh_resources);
    world.insert(lights_resources);
    world.insert(scene_base_resources);
    world.insert(material_resources);

    world.insert(CommandQueue::<RenderMeshCommand, RenderBatch>::new());
    world.insert(CommandQueue::<RenderShadowMeshCommand, RenderShadowBatch>::new());

    world.insert(deferred_pass);
    world.insert(composition_pass);
    world.insert(shadow_passes);
    world.insert(ssao_pass);

    world.insert(RendererEvent::None);

    world.insert(DeltaTimer::new(
        Duration::from_millis(0),
        Instant::now()
    ));

    renderer
}