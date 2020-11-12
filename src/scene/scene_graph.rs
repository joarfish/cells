use specs::prelude::*;
use specs::Component;

#[derive(Component)]
struct Visible;

#[derive(Component)]
struct ModelToWorld {
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
struct Parent(Option<Entity>);

struct TeeNode {
    id: u32,
    parent: u32
}

/// The Scene Graph represents the hierarchical structure of the scene objects.
/// Each entity can be parented to another one.
struct SceneGraph {
    parents_reader: Option<ReaderId<ComponentEvent>>,
    root_node: Option<Entity>,
    node_to_children: std::collections::HashMap<Entity, std::vec::Vec<Entity>>
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
        WriteStorage<'a, ModelToWorld>
    );


    fn run(&mut self, (entities, parents, transformations, modelToWorlds): Self::SystemData) {

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

        for ( entity, transform) in (&entities, &transformations).join() {

        }

        // Compute visibility?

        // Compute render order?

    }

    fn setup(&mut self, world: &mut World) {
        // Setup initial tree
    }

}