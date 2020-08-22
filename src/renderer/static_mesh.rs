use specs::prelude::*;

use super::base::Vertex;
use super::material_types::{Material, RenderObjectType};

pub struct StaticMeshMaterial {
    render_pipeline: wgpu::RenderPipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl StaticMeshMaterial {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vs_src = include_str!("../assets/normal.vert");
        let fs_src = include_str!("../assets/normal.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(vs_spirv.as_binary()),
        ));
        let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(fs_spirv.as_binary()),
        ));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: std::borrow::Cow::Borrowed(&[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }]),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: std::borrow::Cow::Borrowed(&[&uniform_bind_group_layout]),
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
                clamp_depth: false,
            }),
            color_states: std::borrow::Cow::Borrowed(&[wgpu::ColorStateDescriptor {
                format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }]),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: std::borrow::Cow::Borrowed(&[Vertex::desc()]),
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        StaticMeshMaterial {
            render_pipeline,
            uniform_bind_group_layout,
        }
    }
}

impl Material for StaticMeshMaterial {
    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    fn get_uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_bind_group_layout
    }
}

pub struct StaticMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl StaticMesh {
    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer) -> Self {
        StaticMesh {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn vertex_buffer(&self) -> wgpu::BufferSlice {
        self.vertex_buffer.slice(..)
    }

    pub fn index_buffer(&self) -> wgpu::BufferSlice {
        self.index_buffer.slice(..)
    }

    pub fn object_type(&self) -> RenderObjectType {
        RenderObjectType::StaticMesh
    }

    pub fn get_vertices_count(&self) -> u32 {
        3
    }

    pub fn get_indices_count(&self) -> u32 {
        3
    }
}

impl Component for StaticMesh {
    type Storage = VecStorage<Self>;
}
