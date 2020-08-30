use imgui::*;
use imgui_wgpu::Renderer as ImGUIRenderer;
use imgui_winit_support;

pub struct GUI {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl GUI {
    pub fn setup(window: &winit::window::Window,
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
            sc_desc.format
        );

        GUI {
            imgui,
            platform,
            renderer
        }
    }

    pub fn get(&mut self) -> (&mut imgui::Context, &mut imgui_winit_support::WinitPlatform, &mut imgui_wgpu::Renderer) {
        (&mut self.imgui, &mut self.platform, &mut self.renderer)
    }
}

pub struct GUItWrapper{
    gui: *mut GUI
}
    
impl GUItWrapper{

    pub fn new(gui: &mut GUI) -> Self {
        GUItWrapper {
            gui: gui as *mut GUI
        }
    }

    pub fn get(&self) -> &mut GUI
    {
        unsafe{
            &mut *self.gui
        }
    }
}
unsafe impl Sync for GUItWrapper{}
unsafe impl Send for GUItWrapper{}