use std::vec::Vec;

struct BindGroupLayoutHandle {
    index: usize
}

pub struct BindGroupLayoutPool {
    bind_group_layouts: Vec<wgpu::BindGroupLayout>
}

impl BindGroupLayoutPool {
    pub fn new() -> Self {
        BindGroupLayoutPool {
            bind_group_layouts: vec![]
        }
    }

    pub fn add_layout(&mut self, layout: wgpu::BindGroupLayout) -> BindGroupHandle {
        self.bind_group_layouts.push(layout);

        BindGroupLayoutHandle {
            index: self.bind_group_layouts.len() - 1
        }
    }
}

struct BindGroupHandle {
    index: usize,
    generation: usize,
}

pub struct BindGroupPool {
    buffers: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>
}

impl BindGroupPool {
    pub fn new() -> Self {
        BindGroupPool {
            buffers: vec![],
            bind_groups: vec![]
        }
    }

    pub fn add_bind_group_layout(&mut self, layout: wgpu::BindGroupLayout) -> usize {
        self.bind_group_layouts.push(layout);
    }

    pub fn add_bind_group(&mut self, buffer: wgpu::Buffer, bind_group: wgpu::BindGroup, layout_handle: usize) -> BindGroupHandle {
        self.buffers.push(buffer);
        self.bind_groups.push(bind_group);

        BindGroupHandle {
            index: self.buffers.len() - 1,
            generation: 0,
        }
    }
}
