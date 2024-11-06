//====================================================================

use std::sync::Arc;

use camera::Camera;
use pipelines::TexturePipeline;
use shared::SharedRenderResources;
use texture::Texture;
use texture_storage::{DefaultTexture, LoadedTexture};

use super::{tools::Size, window::Window};

pub mod camera;
pub mod pipelines;
pub mod shared;
pub mod texture;
pub mod texture_storage;
pub mod tools;

//====================================================================

pub struct Renderer {
    core: RendererCore,
    shared: SharedRenderResources,
    depth_texture: Texture,
    pub default_texture: DefaultTexture,

    pub camera: Camera,
    pub clear_color: wgpu::Color,

    pub texture_pipeline: TexturePipeline,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let core = pollster::block_on(RendererCore::new(window));
        let shared = SharedRenderResources::new(&core.device);

        let depth_texture = Texture::create_depth_texture(
            &core.device,
            match window.size() {
                Size { width: 0, .. } | Size { height: 0, .. } => Size::new(450, 400),
                _ => window.size(),
            },
            "Depth Texture",
        );

        let default_texture = DefaultTexture::new(Arc::new(LoadedTexture::load_texture(
            &core.device,
            &shared,
            Texture::from_color(
                &core.device,
                &core.queue,
                [255; 3],
                Some("Default Texture"),
                None,
            ),
        )));

        let camera = Camera::new(&core.device, camera::PerspectiveCamera::default());

        let clear_color = wgpu::Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.,
        };

        let texture_pipeline = TexturePipeline::new(
            &core.device,
            &core.config,
            &shared,
            camera.bind_group_layout(),
        );

        Self {
            core,
            shared,
            depth_texture,
            default_texture,
            camera,
            clear_color,
            texture_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: Size<u32>) {
        self.core.config.width = new_size.width;
        self.core.config.height = new_size.height;
        self.core
            .surface
            .configure(&self.core.device, &self.core.config);

        self.depth_texture =
            Texture::create_depth_texture(&self.core.device, new_size, "Depth Texture");
    }

    #[inline]
    pub fn tick(&mut self) {
        self.update();
        self.render();
    }

    fn update(&mut self) {
        self.camera.update_camera(&self.core.queue);

        self.texture_pipeline
            .prep(&self.core.device, &self.core.queue);
    }

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

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render stuff here
        self.texture_pipeline
            .render(&mut render_pass, self.camera.bind_group());

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

        let size = match window.size() {
            Size { width: 0, .. } | Size { height: 0, .. } => Size::new(450, 400),
            _ => window.size(),
        };

        log::debug!("Window inner size = {:?}", size);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    #[cfg(target_arch = "wasm32")]
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    ..Default::default()
                },
                None,
            )
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
