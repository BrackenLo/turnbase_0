//====================================================================

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::tools::Transform;

use super::{
    shared::{SharedRenderResources, Vertex},
    texture_storage::LoadedTexture,
    tools,
};

//====================================================================

pub struct TexturePipeline {
    pipeline: wgpu::RenderPipeline,
    instances: HashMap<u32, TextureInstanceBuffer>,

    previous: HashSet<u32>,
    textures_to_add: HashMap<u32, Arc<LoadedTexture>>,
    to_draw: HashMap<u32, Vec<InstanceTexture>>,
}

impl TexturePipeline {
    pub(super) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedRenderResources,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let pipeline = tools::create_pipeline(
            device,
            config,
            "Texture Pipeline",
            &[camera_bind_group_layout, shared.texture_bind_group_layout()],
            &[InstanceTexture::desc()],
            include_str!("shaders/texture.wgsl"),
            tools::RenderPipelineDescriptor {
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                ..Default::default()
            }
            .with_depth_stencil(),
        );

        let instances = HashMap::default();

        Self {
            pipeline,
            instances,
            previous: HashSet::default(),
            to_draw: HashMap::default(),
            textures_to_add: HashMap::default(),
        }
    }

    pub fn draw_texture(
        &mut self,
        texture: &Arc<LoadedTexture>,
        size: glam::Vec2,
        color: [f32; 4],
        transform: &Transform,
    ) {
        let instance = InstanceTexture {
            // common: InstanceCommon {
            //     transform: transform.to_matrix(),
            //     color: color.into(),
            // },
            size,
            pad: [0.; 2],
            transform: transform.to_matrix(),
            color: color.into(),
        };

        self.to_draw
            .entry(texture.id())
            .or_insert_with(|| {
                self.textures_to_add.insert(texture.id(), texture.clone());
                Vec::new()
            })
            .push(instance);
    }

    pub(super) fn prep(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut new_previous = HashSet::new();

        self.to_draw.drain().for_each(|(id, raw)| {
            new_previous.insert(id);
            self.previous.remove(&id);

            self.instances
                .entry(id)
                .and_modify(|instance| instance.update(device, queue, raw.as_slice()))
                .or_insert(TextureInstanceBuffer::new(
                    device,
                    self.textures_to_add.remove(&id).unwrap(),
                    raw.as_slice(),
                ));
        });

        self.previous.iter().for_each(|to_remove| {
            self.instances.remove(to_remove);
        });

        self.previous = new_previous;
    }

    pub(super) fn render(
        &mut self,
        pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);

        self.instances.iter().for_each(|(_, instance)| {
            pass.set_bind_group(1, instance.texture.bind_group(), &[]);
            pass.set_vertex_buffer(0, instance.buffer.buffer().slice(..));
            pass.draw(0..4, 0..instance.buffer.count());
        });
    }
}

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct InstanceTexture {
    pub size: glam::Vec2,
    pub pad: [f32; 2],
    pub transform: glam::Mat4,
    pub color: glam::Vec4,
}

impl Vertex for InstanceTexture {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x4, // Transform
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4, // Color
            5 => Float32x4, // Size
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

struct TextureInstanceBuffer {
    texture: Arc<LoadedTexture>,
    buffer: tools::InstanceBuffer<InstanceTexture>,
}

impl TextureInstanceBuffer {
    #[inline]
    pub fn new(
        device: &wgpu::Device,
        texture: Arc<LoadedTexture>,
        data: &[InstanceTexture],
    ) -> Self {
        Self {
            texture,
            buffer: tools::InstanceBuffer::new(device, data),
        }
    }

    #[inline]
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[InstanceTexture]) {
        self.buffer.update(device, queue, data);
    }
}

//====================================================================
