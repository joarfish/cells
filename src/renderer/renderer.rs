use specs::prelude::*;
use imgui::{ Condition, im_str };
use wgpu::util::DeviceExt;
use std::time::{Duration, Instant};

use super::uniforms::Uniforms;
use super::static_mesh::StaticMesh;
use super::camera::{ ActiveCamera, Camera };
use super::gui::{ GUItWrapper, GUI };
use super::base::DeltaTimer;
use super::render_state::RendererState;

pub struct Renderer;

pub enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

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

            /* Objects */

            {
                for object in objects.join() {
                    let object_type = object.object_type();
                    render_pass.set_pipeline(object_types.get_pipeline(object_type));
                    render_pass.set_bind_group(0, &uniform_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, object.vertex_buffer());
                    render_pass.set_index_buffer(object.index_buffer());
                    render_pass.draw_indexed(0..object.get_indices_count(), 0, 0..1)
                }
            }

            /* imgui */

            let (imgui, platform, renderer) = gui.get();
            platform.prepare_frame(imgui.io_mut(), &window)
                        .expect("Failed to prepare frame");

            imgui.io_mut().update_delta_time(Instant::now() - Duration::from_millis(16));

            let ui = imgui.frame();
            {
                let window = imgui::Window::new(im_str!("Statistics"));
                window
                    .size([200.0, 100.0], Condition::FirstUseEver)
                    .position([0.0, 0.0], Condition::FirstUseEver)
                    .build(&ui, || {
                        ui.text(im_str!("Frametime: {}", 10));
                    });
            }
            
            if last_cursor != &ui.mouse_cursor() {
                *last_cursor = ui.mouse_cursor();
                platform.prepare_render(&ui, &window);
            }
            
            renderer
            .render(ui.render(), queue, device, &mut render_pass)
            .expect("Rendering failed");
        }

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



impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, RendererEvent>,
        WriteExpect<'a, DeltaTimer>,
        WriteExpect<'a, RendererState>,
        ReadStorage<'a, Camera>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, StaticMesh>,
        ReadExpect<'a, GUItWrapper>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event, mut d_t, mut state, cameras, active_camera, static_meshes, gui) = data;

        match *event {
            RendererEvent::Render => {

                if let Some(camera) = cameras.get((*active_camera).0) {
                    Self::render(&mut state, &static_meshes, &camera, gui.get());
                }

                *event = RendererEvent::None;
                *d_t = DeltaTimer::new(Instant::now() - d_t.get_last_render(), Instant::now());
            }
            RendererEvent::Resize(size) => {
                Self::resize(&mut state, size);
                *event = RendererEvent::None;
            }
            _ => (),
        }
    }
}