pub mod scene_graph;
pub mod spawning;

use specs::prelude::*;

use scene_graph::SceneGraph;

use self::scene_graph::Parent;

pub fn setup_scene(world: &mut specs::World) {
    world.register::<Parent>();

    
}