use std::collections::{BinaryHeap};

/**
    These structure allow us to sort render commands efficiently into
    batches of instanced draw calls. Each Command is encoded as a u32
    and inserted into a binary heap. The renderer can then collect
    the commands into batches, where each batch is of a certain mesh type.
    Within each batch, the instances are sorted with regard to their distance
    to the camera.
*/

/**
    32bit:
    mesh_type (7bit) | material (5bit) | object_index (10bit) | order (10bit) |
*/

#[derive(Clone)]
pub struct RenderMeshCommand {
    pub mesh_type: u8,
    pub material: u8,
    pub object_index: u16,
    pub order: u16,
}

impl Command for RenderMeshCommand {
    fn is_compatible(&self, other: &Self) -> bool {
        self.mesh_type == other.mesh_type && self.material == other.material
    }
}

impl From<u32> for RenderMeshCommand {
    fn from(other: u32) -> Self {
        RenderMeshCommand {
            mesh_type: ((0b1111_1110_0000_0000_0000_0000_0000_0000 & other) >> 25) as u8,
            material: ((0b0000_0001_1111_0000_0000_0000_0000_0000 & other) >> 20) as u8,
            object_index: ((0b0000_0000_0000_1111_1111_1100_0000_0000 & other) >> 10) as u16,
            order: (0b0000_0000_0000_0000_0000_0011_1111_1111 & other) as u16,
        }
    }
}

impl Into<u32> for RenderMeshCommand {
    fn into(self) -> u32 {
        (self.mesh_type as u32) << 25 |
        (self.material as u32) << 20 |
        (self.object_index as u32) << 10 |
        (self.order as u32)
    }
}

pub struct RenderBatch {
    pub object_indices: Vec<u32>,
    pub mesh_type: u16,
    pub material: u8
}

impl Batch<RenderMeshCommand> for RenderBatch {
    fn new(first_command: RenderMeshCommand) -> Self {
        RenderBatch {
            object_indices: vec![first_command.object_index as u32],
            mesh_type: first_command.mesh_type as u16,
            material: first_command.material
        }
    }

    fn add_command(&mut self, command: &RenderMeshCommand) -> bool {
        if command.mesh_type == self.mesh_type as u8 && command.material == self.material {
            if !self.object_indices.contains(&(command.object_index as u32)) {
                self.object_indices.push(command.object_index as u32);
            }
            true
        } else {
            false
        }
    }
}

///////////////////////

pub trait Batch<T: Command> {
    fn new(first_command: T) -> Self;
    fn add_command(&mut self, command: &T) -> bool;
}

pub trait Command: Clone {
    fn is_compatible(&self, other: &Self) -> bool;
}

pub struct CommandQueue<T: From<u32> + Into<u32> + Command, B: Batch<T>> {
    queue: std::collections::BinaryHeap<u32>,
    _marker_command: std::marker::PhantomData<T>,
    _marker_batch: std::marker::PhantomData<B>
}

impl<T: From<u32> + Into<u32> + Command, B: Batch<T>> CommandQueue<T, B> {
    pub fn new() -> Self {
        CommandQueue {
            queue: std::collections::BinaryHeap::new(),
            _marker_command: std::marker::PhantomData::default(),
            _marker_batch: std::marker::PhantomData::default()
        }
    }

    pub fn enqueue_command(&mut self, command: T) {
        self.queue.push(command.into());
    }

    fn pop_command(&mut self) -> Option<T> {
        self.queue.pop().map(|command| {
            T::from(command)
        })
    }

    pub fn pop_next_batch(&mut self) -> Option<B> {

        if let Some(first_command) = self.queue.pop().map(|c| { T::from(c) }) {
            let mut batch = B::new(first_command);

            loop {
                if let Some(command) = self.queue.peek().map(|c| { T::from(*c) }) {
                    if batch.add_command(&command) {
                        self.queue.pop();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Some(batch)
        } else {
            None
        }
    }
}


