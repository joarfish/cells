use super::meshes::Mesh;

pub struct RenderMeshCommand {
    pub mesh: Mesh,
    pub distance: u8
}

impl From<u32> for RenderMeshCommand {
    fn from(other: u32) -> Self {
        RenderMeshCommand {
            mesh: Mesh {
                pool_index: ((0b1111_0000_0000_0000_0000_0000_0000_0000 & other) >> 28) as u16,
                geometry_index: ((0b0000_1111_1111_0000_0000_0000_0000_0000 & other) >> 20) as u32,
                object_index: (0b0000_0000_0000_0000_1111_1111_1111_1111 & other) as u32,
            },
            distance: ((0b0000_0000_0000_1111_0000_0000_0000_0000 & other) >> 16) as u8,
        }
    }
}

impl Into<u32> for RenderMeshCommand {
    fn into(self) -> u32 {
        (self.mesh.pool_index as u32) << 28 |
        (self.mesh.geometry_index << 20) | 
        (self.distance as u32) << 16 | 
        self.mesh.object_index
    }
}

pub struct CommandQueue<T: From<u32> + Into<u32>> {
    queue: std::collections::BinaryHeap<u32>,
    _marker: std::marker::PhantomData<T>
}

impl<T: From<u32> + Into<u32>> CommandQueue<T> {
    pub fn new() -> Self {
        CommandQueue {
            queue: std::collections::BinaryHeap::new(),
            _marker: std::marker::PhantomData::default()
        }
    }

    pub fn enqueue_command(&mut self, command: T) {
        self.queue.push(command.into());
    }

    pub fn pop_command(&mut self) -> Option<T> {
        self.queue.pop().map(|command| {
            T::from(command)
        })
    }
}