//====================================================================

use std::collections::{HashMap, HashSet};

use common::Transform;
use cosmic_text::{Metrics, Wrap};

use crate::{
    shared::Vertex,
    text_shared::{TextAtlas, TextBuffer, TextBufferDescriptor, TextResources, TextVertex},
    texture::Texture,
    tools,
};

//====================================================================

#[derive(Debug, Clone)]
pub struct Ui3d {
    id: u32,
    pub menu_color: [f32; 4],
    pub selection_color: [f32; 4],

    pub options: Vec<String>,
    pub selected: u8,
    pub font_size: f32,
    pub transform: Transform,
}

struct Ui3dData {
    ui_uniform_buffer: wgpu::Buffer,
    ui_uniform_bind_group: wgpu::BindGroup,

    ui_position_uniform_buffer: wgpu::Buffer,
    ui_position_uniform_bind_group: wgpu::BindGroup,
    size: [f32; 2],

    text_buffer: TextBuffer,
}

//====================================================================

pub struct Ui3dRenderer {
    ui_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,

    ui_uniform_bind_group_layout: wgpu::BindGroupLayout,
    ui_position_uniform_bind_group_layout: wgpu::BindGroupLayout,

    current_id: u32,
    to_draw: HashSet<u32>,
    previous: HashSet<u32>,
    instances: HashMap<u32, Ui3dData>,
}

impl Ui3dRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        text_atlas: &TextAtlas,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let ui_position_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ui Instance Buffer Bind Group Layout"),
                entries: &[tools::bgl_uniform_entry(0, wgpu::ShaderStages::VERTEX)],
            });

        let ui_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ui Instance Buffer Bind Group Layout"),
                entries: &[tools::bgl_uniform_entry(0, wgpu::ShaderStages::VERTEX)],
            });

        let ui_pipeline = tools::create_pipeline(
            device,
            config,
            "Ui Renderer",
            &[
                camera_bind_group_layout,
                &ui_uniform_bind_group_layout,
                &ui_position_uniform_bind_group_layout,
            ],
            &[],
            include_str!("shaders/ui3d.wgsl"),
            tools::RenderPipelineDescriptor {
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                fragment_targets: Some(&[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })]),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        write_mask: u32::MAX,
                        ..Default::default()
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                ..Default::default()
            }
            .with_backface_culling(),
        );

        let text_pipeline = tools::create_pipeline(
            device,
            config,
            "Ui Text Renderer",
            &[
                camera_bind_group_layout,
                text_atlas.bind_group_layout(),
                &ui_position_uniform_bind_group_layout,
            ],
            &[TextVertex::desc()],
            include_str!("shaders/text.wgsl"),
            tools::RenderPipelineDescriptor {
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                fragment_targets: Some(&[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })]),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        write_mask: u32::MAX,
                        ..Default::default()
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                ..Default::default()
            },
        );

        Self {
            ui_pipeline,
            text_pipeline,
            ui_uniform_bind_group_layout,
            ui_position_uniform_bind_group_layout,
            current_id: 0,
            previous: HashSet::default(),
            to_draw: HashSet::default(),
            instances: HashMap::default(),
        }
    }

    pub fn create_ui(&mut self, options: Vec<String>) -> Ui3d {
        let id = self.current_id;
        self.current_id += 1;

        Ui3d {
            id,
            menu_color: [0.5, 0.5, 0.5, 0.7],
            selection_color: [0.7, 0.7, 0.7, 0.8],
            options,
            selected: 0,
            font_size: 30.,
            transform: Transform::from_scale((0.3, 0.3, 0.3)),
        }
    }

    // Prep text
    pub fn draw_ui(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_res: &mut TextResources,
        ui: &Ui3d,
    ) {
        if !self.instances.contains_key(&ui.id) {
            self.insert_ui(device, &mut text_res.font_system, ui);
        }

        let data = self.instances.get_mut(&ui.id).unwrap();

        let position_raw = UiPositionUniformRaw {
            transform: ui.transform.to_matrix(),
        };

        queue
            .write_buffer_with(
                &data.ui_position_uniform_buffer,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<UiPositionUniformRaw>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[position_raw]));

        let longest_line = ui.options.iter().reduce(|a, b| match a.len() < b.len() {
            true => a,
            false => b,
        });

        let longest_line = match longest_line {
            Some(val) => val,
            None => return,
        };

        let selected = ui.selected.clamp(0, ui.options.len() as u8) as f32;

        let option_count = ui.options.len() as f32;
        let option_range = 1. / option_count;

        let ui_size = glam::vec2(
            ui.font_size * longest_line.len() as f32,
            ui.font_size * option_count,
        );

        let ui_raw = UiUniformRaw {
            size: ui_size,
            menu_color: ui.menu_color.into(),
            selection_color: ui.selection_color.into(),
            selection_range_y: glam::vec2(option_range * selected, option_range * (selected + 1.)),

            pad: [0.; 2],
            pad2: [0.; 2],
        };

        queue
            .write_buffer_with(
                &data.ui_uniform_buffer,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<UiUniformRaw>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[ui_raw]));

        data.size = ui_size.to_array();

        data.text_buffer.set_metrics(
            &mut text_res.font_system,
            Metrics::new(ui.font_size, ui.font_size),
        );

        if let Some(rebuild) = crate::text_shared::prep(
            device,
            queue,
            &mut text_res.font_system,
            &mut text_res.swash_cache,
            &mut text_res.text_atlas,
            &mut data.text_buffer,
        ) {
            tools::update_instance_buffer(
                device,
                queue,
                "UI3d Text Vertex Buffer",
                &mut data.text_buffer.vertex_buffer,
                &mut data.text_buffer.vertex_count,
                &rebuild,
            );
        }

        self.to_draw.insert(ui.id);
    }

    fn insert_ui(
        &mut self,
        device: &wgpu::Device,
        font_system: &mut cosmic_text::FontSystem,
        ui: &Ui3d,
    ) {
        log::trace!("Preparing new ui3d");

        let ui_uniform_buffer = tools::buffer(
            device,
            tools::BufferType::Uniform,
            "Ui",
            &[UiUniformRaw {
                size: glam::vec2(1., 1.),
                pad: [0.; 2],
                menu_color: glam::vec4(1., 1., 1., 1.),
                selection_color: glam::vec4(1., 0., 0., 1.),
                selection_range_y: glam::vec2(0., 0.),
                pad2: [0.; 2],
            }],
        );

        let ui_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ui Bind Group"),
            layout: &self.ui_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    ui_uniform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let ui_position_uniform_buffer = tools::buffer(
            device,
            tools::BufferType::Uniform,
            "Ui Position",
            &[UiPositionUniformRaw {
                transform: glam::Mat4::default(),
            }],
        );

        let ui_position_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ui Position Bind Group"),
            layout: &self.ui_position_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    ui_position_uniform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let text = ui
            .options
            .iter()
            .cloned()
            .reduce(|a, b| format!("{}\n{}", a, b))
            .unwrap_or(String::new());

        let text_buffer = TextBuffer::new(
            device,
            font_system,
            &TextBufferDescriptor {
                metrics: Metrics::new(10., 10.),
                word_wrap: Wrap::None,
                // attributes: todo!(),
                text: &text,
                // width: todo!(),
                // height: todo!(),
                // color: todo!(),
                ..Default::default()
            },
        );

        self.instances.insert(
            ui.id,
            Ui3dData {
                ui_uniform_buffer,
                ui_uniform_bind_group,
                ui_position_uniform_buffer,
                ui_position_uniform_bind_group,
                size: [1., 1.],
                text_buffer,
            },
        );
    }

    pub(crate) fn prep(&mut self) {
        self.to_draw.iter().for_each(|id| {
            self.previous.remove(id);
        });

        self.previous.iter().for_each(|id| {
            self.instances.remove(id);
        });

        self.previous = self.to_draw.drain().collect::<HashSet<_>>();
    }

    pub(crate) fn render(
        &self,
        pass: &mut wgpu::RenderPass,
        text_atlas: &TextAtlas,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        // Set camera (both pipelines)
        pass.set_bind_group(0, camera_bind_group, &[]);

        // Draw UI background
        pass.set_pipeline(&self.ui_pipeline);

        self.instances.values().into_iter().for_each(|instance| {
            pass.set_bind_group(1, &instance.ui_uniform_bind_group, &[]);
            pass.set_bind_group(2, &instance.ui_position_uniform_bind_group, &[]);
            pass.draw(0..4, 0..1);
        });

        // Draw Text
        pass.set_pipeline(&self.text_pipeline);
        pass.set_bind_group(1, text_atlas.bind_group(), &[]);

        self.instances.values().into_iter().for_each(|instance| {
            pass.set_vertex_buffer(0, instance.text_buffer.vertex_buffer.slice(..));
            pass.set_bind_group(2, &instance.ui_position_uniform_bind_group, &[]);
            pass.draw(0..4, 0..instance.text_buffer.vertex_count);
        });
    }
}

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct UiPositionUniformRaw {
    transform: glam::Mat4,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct UiUniformRaw {
    pub size: glam::Vec2,
    pub pad: [f32; 2],

    pub menu_color: glam::Vec4,
    pub selection_color: glam::Vec4,
    pub selection_range_y: glam::Vec2,
    pub pad2: [f32; 2],
}

//====================================================================
