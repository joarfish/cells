use specs::prelude::*;
use specs::Component;

use crate::renderer::command_queue::{CommandQueue, RenderMeshCommand, RenderBatch};

use crate::renderer::shadow_passes::{RenderShadowMeshCommand, RenderShadowBatch};
use crate::scene::solid_object::SolidObject;

#[derive(Component)]
pub struct Visible;

#[derive(Component)]
pub struct ModelToWorld {
    transform: cgmath::Matrix4<f32>
}

#[derive(Component)]
pub struct Transformation {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Euler<cgmath::Deg<f32>>,
    pub scale: cgmath::Point3<f32>
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Parent(Option<Entity>);

struct TeeNode {
    id: u32,
    parent: u32
}

/// The Scene Graph represents the hierarchical structure of the scene objects.
/// Each entity can be parented to another one.
pub struct SceneGraph {
    parents_reader: Option<ReaderId<ComponentEvent>>,
    root_node: Option<Entity>,
    node_to_children: std::collections::HashMap<Entity, std::vec::Vec<Entity>>
}

impl Default for SceneGraph {
    fn default() -> Self {
        SceneGraph {
            parents_reader: None,
            root_node: None,
            node_to_children: std::collections::HashMap::new()
        }
    }
}

impl SceneGraph {

    fn add_node(&mut self, entity: Entity, parent: Option<Entity>) {

        if let Some(parent) = parent {
            match self.node_to_children.get_mut(&parent) {
                Some(children) => {
                    children.push(entity);
                }
                None => {
                    self.node_to_children.insert( parent, vec![ entity ]);
                }
            }
        } else {
            if self.root_node.is_some() {
                panic!("Scene Graph already has a root node!");
            }
            self.root_node = Some(entity);
        }
    }

}

impl<'a> System<'a> for SceneGraph {

    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Parent>,
        ReadStorage<'a, Transformation>,
        ReadStorage<'a, SolidObject>,
        WriteExpect<'a, CommandQueue<RenderMeshCommand, RenderBatch>>,
        WriteExpect<'a, CommandQueue<RenderShadowMeshCommand, RenderShadowBatch>>
    );

    fn run(&mut self, data: Self::SystemData) {

        if self.parents_reader.is_none() {
            return;
        }

        let (
            entities,
            parents,
            transformations,
            solid_objects,
            mut commands_queue,
            mut shadow_commands_queue,
        ) = data;

        let events = parents
            .channel()
            .read(self.parents_reader.as_mut().unwrap());

        // Process parenting updates:

        let mut inserted : BitSet = BitSet::new();
        let mut updated : BitSet = BitSet::new();
        let mut removed : BitSet = BitSet::new();

        for event in events {
            match event {
                ComponentEvent::Inserted(id) => {
                    inserted.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    updated.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    removed.add(*id);
                }
            }
        }

        for (entity, parent, _) in (&entities, &parents, &inserted).join() {
            if let Some(parent_entity) = parent.0 {
                // add entity to graph with parent_entity as its... well... parent
            }
        }

        for (entity, parent, _) in (&entities, &parents, &updated).join() {
            if let Some(parent_entity) = parent.0 {
                //modify graph: E.g. remove children and children of children...
            }
        }

        for (entity, parent, _) in (&entities, &parents, &removed).join() {

        }

        // Traverse tree and update transforms

        for (entity, transform) in (&entities, &transformations).join() {

        }

        // Compute visibility?

        // Compute render order?

        for solid_object in (&solid_objects).join() {
            commands_queue.enqueue_command(RenderMeshCommand {
                mesh_type: solid_object.mesh_type as u8,
                material: solid_object.material as u8,
                object_index: solid_object.object_index as u16,
                order: 1
            });
            shadow_commands_queue.enqueue_command(RenderShadowMeshCommand {
                mesh_type: solid_object.mesh_type as u8,
                object_index: solid_object.object_index as u16,
                order: 1
            });
        }

    }

    fn setup(&mut self, world: &mut World) {
        log::info!("Setup on SceneGraph");
        Self::SystemData::setup(world);
        self.parents_reader = Some(
            WriteStorage::<Parent>::fetch(&world).register_reader()
        );
    }

}