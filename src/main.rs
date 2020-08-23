mod input;
mod renderer;

use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use wgpu::Instance;
use winit::event;
use winit::event::KeyboardInput;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use specs::prelude::*;
use std::borrow::BorrowMut;
use renderer::base::Vertex;
use renderer::camera::*;
use renderer::material_types::{Material, MaterialTypes, RenderObjectType};
use renderer::static_mesh::{StaticMesh, StaticMeshMaterial};
use renderer::uniforms::Uniforms;
use input::InputMap;

use imgui::*;
use imgui_wgpu::Renderer as ImGUIRenderer;
use imgui_winit_support;

pub struct DeltaTimer {
    d: Duration,
    last_render: Instant,
}

impl DeltaTimer {
    pub fn get_duration(&self) -> Duration {
        self.d
    }

    pub fn get_duration_ms(&self) -> u32 {
        self.d.as_millis() as u32
    }

    pub fn get_duration_f32(&self) -> f32 {
        self.d.as_secs_f32()
    }
}

struct RendererState {
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    object_types: MaterialTypes,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    window: winit::window::Window,
    last_cursor: Option<MouseCursor>
}

impl RendererState {
    async fn new(window: winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        // Todo: Specify required features
        let adapter_features = adapter.features();
        log::info!("Features: {:?}", adapter_features);

        // Todo: Specify limits
        let limits = wgpu::Limits {
            max_bind_groups: 6,
            ..wgpu::Limits::default()
        };
        log::info!("Limits: {:#?}", limits);

        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, mut queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: limits,
                    shader_validation: true,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let object_types = MaterialTypes::new(&device, sc_desc.format);

        /* Create Uniforms */
        let uniforms = Uniforms::new();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[uniforms]),
            label: None,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(0..)),
            }],
            layout: object_types.get_uniform_layout(RenderObjectType::StaticMesh),
        });

        RendererState {
            instance,
            adapter,
            size,
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            object_types,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            window,
            last_cursor: None,
        }
    }
}

struct GUI {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl GUI {
    fn setup(window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor
    ) -> Self {
        // ImGUI

        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = 1.0;

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        //
        // Set up dear imgui wgpu renderer
        //

        let renderer = ImGUIRenderer::new(
            &mut imgui,
            device,
            queue,
            sc_desc.format,
            None,
        );

        GUI {
            imgui,
            platform,
            renderer
        }
    }
}

struct Renderer;

impl Renderer {
    fn render<'a>(
        RendererState {
            device,
            swap_chain,
            queue,
            object_types,
            uniforms,
            uniform_bind_group,
            uniform_buffer,
            window,
            last_cursor,
            ..
        }: &'a mut RendererState,
        objects: &ReadStorage<'a, StaticMesh>,
        camera: &Camera,
        gui: &mut GUI
    ) {
        let screen_frame = swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture!");
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            /* Update camera and corresponding uniform: */
            uniforms.update_view_proj(camera);
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[*uniforms]),
                usage: wgpu::BufferUsage::COPY_SRC,
            });

            encoder.copy_buffer_to_buffer(
                &staging_buffer,
                0,
                uniform_buffer,
                0,
                std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            );
            queue.submit(std::iter::once(encoder.finish()));
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &screen_frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            for object in objects.join() {
                let object_type = object.object_type();
                render_pass.set_pipeline(object_types.get_pipeline(object_type));
                render_pass.set_bind_group(0, &uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, object.vertex_buffer());
                render_pass.set_index_buffer(object.index_buffer());
                render_pass.draw_indexed(0..object.get_indices_count(), 0, 0..1)
            }
        }

        queue.submit(std::iter::once(encoder.finish()));

        /* imgui */

        gui.platform.prepare_frame(gui.imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");

        gui.imgui.io_mut().update_delta_time(Instant::now() - Duration::from_millis(16));

        let ui = gui.imgui.frame();
        {
            let window = imgui::Window::new(im_str!("Hello too"));
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([0.0, 0.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Frametime:"));
                });
        }

        let mut encoder: wgpu::CommandEncoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if last_cursor != &ui.mouse_cursor() {
            *last_cursor = ui.mouse_cursor();
            gui.platform.prepare_render(&ui, &window);
        }
        
        gui.renderer
            .render(ui.render(), device, &mut encoder, &screen_frame.output.view)
            .expect("Rendering failed");

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn resize(
        RendererState {
            surface,
            device,
            sc_desc,
            ..
        }: &mut RendererState,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        sc_desc.width = new_size.width;
        sc_desc.height = new_size.height;
        device.create_swap_chain(&surface, &sc_desc);
    }
}

enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, RendererEvent>,
        WriteExpect<'a, DeltaTimer>,
        WriteExpect<'a, RendererState>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, ActiveCamera>,
        ReadStorage<'a, StaticMesh>,
        ReadExpect<'a, GUItWrapper>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event, mut d_t, mut state, cameras, active_cameras, static_meshes, gui) = data;

        match *event {
            RendererEvent::Render => {
                if let Some((camera, _)) = (&cameras, &active_cameras).join().into_iter().nth(0) {
                    Self::render(&mut state, &static_meshes, &camera, gui.get());
                }
                *event = RendererEvent::None;
                *d_t = DeltaTimer {
                    d: Instant::now() - d_t.last_render,
                    last_render: Instant::now(),
                }
            }
            RendererEvent::Resize(size) => {
                Self::resize(&mut state, size);
                *event = RendererEvent::None;
            }
            _ => (),
        }
    }
}

struct GUItWrapper{
    gui: *mut GUI
}
    
impl GUItWrapper{

    pub fn new(gui: &mut GUI) -> Self {
        GUItWrapper {
            gui: gui as *mut GUI
        }
    }

    fn get(&self) -> &mut GUI
    {
        unsafe{
            &mut *self.gui
        }
    }
}
unsafe impl Sync for GUItWrapper{}
unsafe impl Send for GUItWrapper{}

fn main() {
    env_logger::init();

    let mut world = World::new();

    /*let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };*/

    log::info!("Setting things up.");

    let event_loop = EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new();
    let window = window_builder
        .with_title("Cells")
        .build(&event_loop)
        .unwrap();

    let mut renderer_state = futures::executor::block_on(RendererState::new(window));

    let mut gui = GUI::setup(&renderer_state.window, &renderer_state.device, &mut renderer_state.queue, &renderer_state.sc_desc);

    /* Register Components */
    world.register::<StaticMesh>();
    world.register::<Camera>();
    world.register::<ActiveCamera>();

    /* Create Entities */
    world
        .create_entity()
        .with(StaticMesh::new(
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        Vertex::new([0.25, 0.25, 0.0], [1.0, 0.0, 0.0]),
                        Vertex::new([-0.5, -0.25, 0.0], [0.0, 1.0, 0.0]),
                        Vertex::new([0.5, -0.25, 0.0], [0.0, 0.0, 1.0]),
                    ]),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[0 as u16, 1, 2]),
                    usage: wgpu::BufferUsage::INDEX,
                }),
        ))
        .build();

    world
        .create_entity()
        .with(StaticMesh::new(
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        Vertex::new([0.75, 0.75, 0.0], [1.0, 0.0, 0.0]),
                        Vertex::new([0.25, -0.75, 0.0], [0.0, 1.0, 0.0]),
                        Vertex::new([0.99, -0.75, 0.0], [0.0, 0.0, 1.0]),
                    ]),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[0 as u16, 1, 2]),
                    usage: wgpu::BufferUsage::INDEX,
                }),
        ))
        .build();

    world
        .create_entity()
        .with(Camera::new(
            renderer_state.sc_desc.width as f32 / renderer_state.sc_desc.height as f32,
        ))
        .with(ActiveCamera)
        .build();

    /* Add Resources */
    world.insert(RendererEvent::None);
    world.insert(renderer_state);
    world.insert(DeltaTimer {
        d: Duration::from_millis(0),
        last_render: Instant::now(),
    });
    world.insert(InputMap::new());
    world.insert(GUItWrapper::new(&mut gui));

    let mut dispatcher = DispatcherBuilder::new()
        .with(CameraSystem, "Camera System", &[])
        .with_thread_local(Renderer)
        .build();

    let mut last_update = Instant::now();
    let mut last_render = Instant::now();

    //let mut last_cursor = None;

    event_loop.run(move |event, _, control_flow| {
        
        {
            match event {
                event::Event::MainEventsCleared => {
                    let RendererState { window, .. } = &*world.read_resource::<RendererState>();
                    window.request_redraw();
                }
                event::Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    log::info!("Resizing to {:?}", size);
                    let mut cameras = world.write_component::<Camera>();
                    for camera in (&mut cameras).join() {
                        (*camera).resize(size);
                    }
                    let mut renderer_event = world.write_resource::<RendererEvent>();
                    *renderer_event = RendererEvent::Resize(size);
                }
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        log::info!("Closing Application.");
                    },
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state: key_state,
                                ..
                            },
                        ..
                    } => {
                        let mut input_map = world.write_resource::<InputMap>();
                        input_map.update(key_code, key_state);
                    }
                    _ => {}
                },
                event::Event::RedrawRequested(_) => {

                    // Render approx. 60 times a second
                    if (Instant::now() - last_render) >= Duration::from_millis(16) {
                        *world.write_resource::<RendererEvent>() = RendererEvent::Render;
                        last_render = Instant::now();
                    }

                    // Update approx. 250 times per second
                    if (Instant::now() - last_update) >= Duration::from_millis(4) {
                        dispatcher.dispatch(&mut world);
                        world.maintain();
                        last_update = Instant::now();
                    }

                    {
                        let RendererState { window, .. } = &*world.read_resource::<RendererState>();
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }
    });
}
