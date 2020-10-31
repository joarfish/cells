use specs::prelude::*;

use super::base::Vertex;
use super::material_types::{ RenderObjectType};

pub struct Shader<'a> {
    vertex_state_descriptors: Vec<wgpu::VertexBufferDescriptor<'a>>,
    vertex_shader_module: wgpu::ShaderModule,
    fragment_shader_module: wgpu::ShaderModule,
}

pub struct ShaderBuilder<'a> {
    id: u64,
    vertex_attribute_descriptors: Vec<Vec<wgpu::VertexAttributeDescriptor>>,
    vertex_buffer_descriptors: Vec<wgpu::VertexBufferDescriptor<'a>>,
    vertex_attributes_string: String,
    uniform_groups: Vec<wgpu::BindGroupLayoutEntry>,
    free_vs_uniform_set: u32,
    free_vs_attr_location: u32,
    free_fs_attr_location: u32,
}

static LAST_ID: std::sync::Mutex<u64> = std::sync::Mutex::new(0);

impl<'a> ShaderBuilder<'a> {

    pub fn new() -> Self {
        let mut id = LAST_ID.lock().unwrap();
        *id = (*id) + 1;

        ShaderBuilder {
            id: *id,
            vertex_attribute_descriptors: vec![],
            vertex_buffer_descriptors: vec![],
            vertex_attributes_string: "".to_owned(),
            uniform_groups: vec![],
            free_vs_uniform_set: 0,
            free_vs_attr_location: 0,
            free_fs_attr_location: 0,
        }
    }

    pub fn with_view_proj_matrix(self) -> Self {

        self
    }

    pub fn with_vertex_attribute<T : Sized>(mut self, format: wgpu::VertexFormat, attribute_name: &str) -> Self {

        self.vertex_attribute_descriptors.push(vec![ wgpu::VertexAttributeDescriptor {
            offset: 0,
            shader_location: self.free_vs_attr_location,
            format
        } ]);

        let va_desc = (&'a self.vertex_attribute_descriptors).last().unwrap();

        self.vertex_buffer_descriptors.push(
            wgpu::VertexBufferDescriptor {
                attributes: &va_desc,
                step_mode: wgpu::InputStepMode::Vertex,
                stride: format.size() as wgpu::BufferAddress
            }
        );

        let format_str = match format {
            wgpu::VertexFormat::Int => "int",
            wgpu::VertexFormat::Int2 => "ivec2",
            wgpu::VertexFormat::Int3 => "ivec3",
            wgpu::VertexFormat::Int4 => "ivec4",

            wgpu::VertexFormat::Float => "float",
            wgpu::VertexFormat::Float2 => "vec2",
            wgpu::VertexFormat::Float3 => "vec3",
            wgpu::VertexFormat::Float4 => "vec4",
            _ => {
                panic!("Vertex Format not supported yet. (Because Jonas is lazy...)");
            }
        };

        self.vertex_attributes_string.push_str(
            &format!("layout(location={} in {} {};\n", self.free_vs_attr_location, format_str, attribute_name)
        );

        self.free_vs_attr_location+=1;

        self
    }

    pub fn add_uniform_buffer(mut self, stage: wgpu::ShaderStage, block_name: &str) -> Self {

        self.uniform_groups.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: stage,
            ty: wgpu::BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        });


        self
    }


    pub fn build(self, device: wgpu::Device) -> Shader<'a> {
        let vs_code = String::from(
            "
            #version 450

            #attributes

            layout(location=0) out vec3 v_color;

            layout(set=0, binding=0)
            uniform Uniforms {
                mat4 u_view_proj;
            };

            void main() {
                v_color = a_color;
                gl_Position = u_view_proj * vec4(a_position, 1.0);
            }
            "
        );

        vs_code.replace("#attributes", &self.vertex_attributes_string);

        let fs_code = String::from("");

        /*let vertex_state_descriptor = wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &self.vertex_buffer_descriptors,
        };*/

        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_spirv = compiler.compile_into_spirv(
                &vs_code,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_code,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        Shader {
            vertex_state_descriptors: self.vertex_buffer_descriptors,
            vertex_shader_module: device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
                std::borrow::Cow::Borrowed(vs_spirv.as_binary()),
            )),
            fragment_shader_module: device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
                std::borrow::Cow::Borrowed(fs_spirv.as_binary()),
            ))
        }
    }
}

pub struct MaterialBuilder {
    vs_module: Option<wgpu::ShaderModule>,
    fs_module: Option<wgpu::ShaderModule>,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        MaterialBuilder {
            vs_module: None,
            fs_module: None
        }
    }

    pub fn with_vertex_shader_module(mut self, vs_module : wgpu::ShaderModule) -> Self {
        self.vs_module = Some(vs_module);
        self
    }


    pub fn with_fragment_shader_module(mut self, fs_module : wgpu::ShaderModule) -> Self {
        self.fs_module = Some(fs_module);
        self
    }

    pub fn build(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> Material {
        let vs_src = include_str!("../assets/normal.vert");
        let fs_src = include_str!("../assets/normal.frag");

        /*let mut compiler = shaderc::Compiler::new().unwrap();
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
        ));*/

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
                label: None
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &self.vs_module.unwrap(),
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &self.fs_module.unwrap(),
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Material::new(render_pipeline, uniform_bind_group_layout)
    }
}



pub struct Material {
    render_pipeline: wgpu::RenderPipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl Material {
    fn new(render_pipeline: wgpu::RenderPipeline, uniform_bind_group_layout: wgpu::BindGroupLayout) -> Self {
        
        Material {
            render_pipeline,
            uniform_bind_group_layout,
        }
    }

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn get_uniform_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_bind_group_layout
    }
}

pub trait Mesh {
    fn vertex_buffer(&self) -> wgpu::BufferSlice;
    fn index_buffer(&self) -> wgpu::BufferSlice;
    fn get_vertices_count(&self) -> u32;
    fn get_indices_count(&self) -> u32;
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


