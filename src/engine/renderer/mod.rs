//====================================================================

use pollster::FutureExt;

use super::{tools::Size, window::Window};

//====================================================================

pub struct Renderer {
    core: RendererCore,
    clear_color: wgpu::Color,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let clear_color = wgpu::Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.,
        };

        Self {
            core: RendererCore::new(window).block_on(),
            clear_color,
        }
    }

    pub fn resize(&mut self, new_size: Size<u32>) {
        self.core.config.width = new_size.width;
        self.core.config.height = new_size.height;
        self.core
            .surface
            .configure(&self.core.device, &self.core.config);
    }

    #[inline]
    pub fn tick(&mut self) {
        self.update();
        self.render();
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        let mut encoder =
            self.core
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Main command encoder"),
                });

        let (surface_texture, surface_view) = match self.core.surface.get_current_texture() {
            Ok(texture) => {
                let view = texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                (texture, view)
            }
            Err(_) => {
                log::warn!("Unable to get surface texture - skipping frame");
                return;
            }
        };

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render stuff here

        // Finish render pass
        std::mem::drop(render_pass);

        self.core.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

//====================================================================

pub struct RendererCore {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl RendererCore {
    pub async fn new(window: &Window) -> Self {
        log::debug!("Creating core wgpu renderer components.");

        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.0.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        log::debug!("Chosen device adapter: {:#?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        log::debug!("Successfully created core wgpu components.");

        Self {
            device,
            queue,
            surface,
            config,
        }
    }
}

//====================================================================
