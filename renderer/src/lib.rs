//====================================================================

use std::sync::Arc;

use camera::Camera;
use common::Size;
use hecs::World;
use pipelines::{texture_pipeline::TextureRenderer, ui3d_pipeline::Ui3dRenderer};
use shared::SharedRenderResources;
use text_shared::TextResources;
use texture::Texture;
use texture_storage::{DefaultTexture, LoadedTexture};
use wgpu::SurfaceTarget;

pub mod camera;
pub mod pipelines;
pub mod shared;
pub mod text_shared;
pub mod texture;
pub mod texture_storage;
pub mod tools;

//====================================================================

pub struct Renderer {
    core: RendererCore,
    _shared: SharedRenderResources,
    depth_texture: Texture,
    pub default_texture: DefaultTexture,

    pub camera: Camera,
    pub clear_color: wgpu::Color,

    text_res: TextResources,
    texture_pipeline: TextureRenderer,
    ui3d_pipeline: Ui3dRenderer,
}

impl Renderer {
    pub fn new(window: impl Into<SurfaceTarget<'static>>, window_size: Size<u32>) -> Self {
        let core = pollster::block_on(RendererCore::new(window, window_size));
        let shared = SharedRenderResources::new(&core.device);

        let depth_texture =
            Texture::create_depth_texture(&core.device, window_size, "Depth Texture");

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

        let text_res = TextResources::new(&core.device);

        let texture_pipeline = TextureRenderer::new(
            &core.device,
            &core.config,
            &shared,
            camera.bind_group_layout(),
        );

        let ui3d_pipeline = Ui3dRenderer::new(
            &core.device,
            &core.config,
            &text_res.text_atlas,
            camera.bind_group_layout(),
        );

        Self {
            core,
            _shared: shared,
            depth_texture,
            default_texture,
            camera,
            clear_color,
            text_res,
            texture_pipeline,
            ui3d_pipeline,
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
    pub fn tick(&mut self, world: &mut World) {
        self.update(world);
        self.render(world);

        self.core.device.poll(wgpu::Maintain::Wait);

        self.text_res.text_atlas.post_render_trim();
    }

    fn update(&mut self, world: &mut World) {
        self.camera.update_camera(&self.core.queue);

        self.texture_pipeline
            .prep(world, &self.core.device, &self.core.queue);

        self.ui3d_pipeline
            .prep_rotations(world, self.camera.camera.translation);

        self.ui3d_pipeline.prep(
            world,
            &self.core.device,
            &self.core.queue,
            &mut self.text_res,
        );
    }

    fn render(&mut self, _world: &mut World) {
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

        let mut encoder = self
            .core
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.render_inner(&mut encoder, &surface_view);

        self.core.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn render_inner(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) {
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

        self.ui3d_pipeline.render(
            &mut render_pass,
            &self.text_res.text_atlas,
            self.camera.bind_group(),
        );
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
    pub async fn new(window: impl Into<SurfaceTarget<'static>>, window_size: Size<u32>) -> Self {
        log::debug!("Creating core wgpu renderer components.");

        log::debug!("Window inner size = {:?}", window_size);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // let surface = instance.create_surface(window.0.clone()).unwrap();
        let surface = instance.create_surface(window).unwrap();

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
            width: window_size.width,
            height: window_size.height,
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
