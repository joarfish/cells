pub mod scene_graph;
pub mod camera;
pub mod spawning;
pub mod dynamic_objects;
pub mod lights;
pub mod static_objects;

use specs::prelude::*;

use self::{camera::Camera, scene_graph::Parent};
use crate::scene::static_objects::StaticObject;
use crate::scene::scene_graph::Transformation;
use crate::scene::dynamic_objects::{Color, DynamicObject};

pub struct SceneInfo {
    pub dynamic_objects_pool: u32,
    pub static_objects_pool: u32,
}

pub fn setup_scene(world: &mut specs::World) {

    world.insert(SceneInfo {
        dynamic_objects_pool: 0,
        static_objects_pool: 0
    });

    world.register::<Parent>();
    world.register::<Camera>();
    world.register::<StaticObject>();
    world.register::<DynamicObject>();
    world.register::<Transformation>();
    world.register::<Color>();
}