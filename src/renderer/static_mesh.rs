use specs::prelude::*;

use super::render_object_type::RenderObjectType;
use super::base::Vertex;

pub struct StaticMesh {
    render_pipeline: wgpu::RenderPipeline
}

impl StaticMesh {
    pub fn new(device : &wgpu::Device, format : wgpu::TextureFormat) -> Self {
        let vs_src = include_str!("../assets/normal.vert");
        let fs_src = include_str!("../assets/normal.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler.compile_into_spirv(vs_src, shaderc::ShaderKind::Vertex, "shader.vert", "main", None).unwrap();
        let fs_spirv = compiler.compile_into_spirv(fs_src, shaderc::ShaderKind::Fragment, "shader.frag", "main", None).unwrap();

        let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(std::borrow::Cow::Borrowed(vs_spirv.as_binary())));
        let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(std::borrow::Cow::Borrowed(fs_spirv.as_binary())));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: std::borrow::Cow::Borrowed(&[]),
            push_constant_ranges: std::borrow::Cow::Borrowed(&[]),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: std::borrow::Cow::Borrowed("main"),
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: std::borrow::Cow::Borrowed("main"),
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false
            }),
            color_states: std::borrow::Cow::Borrowed(&[
                wgpu::ColorStateDescriptor {
                    format: format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ]),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: std::borrow::Cow::Borrowed(&[
                    Vertex::desc()
                ]),
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        StaticMesh {
            render_pipeline
        }
    }
}

impl RenderObjectType for StaticMesh {
    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}

pub struct StaticMeshObject {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    object_type: usize
}

impl StaticMeshObject {
    pub fn new(vertex_buffer : wgpu::Buffer, index_buffer : wgpu::Buffer) -> Self {
        StaticMeshObject {
            vertex_buffer,
            index_buffer,
            object_type: 0
        }
    }

    pub fn vertex_buffer(&self) -> wgpu::BufferSlice {
        self.vertex_buffer.slice(..)
    }

    pub fn index_buffer(&self) -> wgpu::BufferSlice {
        self.index_buffer.slice(..)
    }

    pub fn object_type(&self) -> usize {
        self.object_type
    }

    pub fn get_vertices_count(&self) -> u32 {
        3
    }

    pub fn get_indices_count(&self) -> u32 {
        3
    }
}

impl Component for StaticMeshObject {
    type Storage = VecStorage<Self>;
}