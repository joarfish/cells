pub mod scene_graph;
pub mod camera;
pub mod spawning;
pub mod lights;
pub mod solid_object;
pub mod playing_field;

use specs::prelude::*;

use self::{camera::Camera, scene_graph::Parent};
use crate::scene::solid_object::SolidObject;
use crate::scene::scene_graph::{Transformation, SceneResources};
use crate::renderer::utils::AABB;

pub fn setup_scene(world: &mut specs::World) {

    world.insert(SceneResources {
        extend: AABB::new(cgmath::Point3::new(-0.5, -0.25, -0.5), cgmath::Point3::new(20.0, 0.0, 20.0))
    });

    world.register::<Parent>();
    world.register::<Camera>();
    world.register::<SolidObject>();
    world.register::<Transformation>();
}