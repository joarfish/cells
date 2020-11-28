pub mod scene_graph;
pub mod camera;
pub mod spawning;
pub mod lights;
pub mod solid_object;
pub mod playing_field;

use specs::prelude::*;

use self::{camera::Camera, scene_graph::Parent};
use crate::scene::solid_object::SolidObject;
use crate::scene::scene_graph::Transformation;

pub fn setup_scene(world: &mut specs::World) {

    world.register::<Parent>();
    world.register::<Camera>();
    world.register::<SolidObject>();
    world.register::<Transformation>();
}