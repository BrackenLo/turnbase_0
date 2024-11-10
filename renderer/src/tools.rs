//====================================================================

use std::{marker::PhantomData, num::NonZeroU32};

use wgpu::util::DeviceExt;

use super::texture::Texture;

//====================================================================

pub struct RenderPipelineDescriptor<'a> {
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState,
    pub fragment_targets: Option<&'a [Option<wgpu::ColorTargetState>]>,
    pub multiview: Option<NonZeroU32>,
    pub cache: Option<&'a wgpu::PipelineCache>,
}

impl<'a> Default for RenderPipelineDescriptor<'a> {
    fn default() -> Self {
        Self {
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment_targets: None,
            multiview: None,
            cache: None,
        }
    }
}

impl RenderPipelineDescriptor<'_> {
    pub fn with_depth_stencil(mut self) -> Self {
        self.depth_stencil = Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        self
    }

    pub fn with_backface_culling(mut self) -> Self {
        self.primitive.cull_mode = Some(wgpu::Face::Back);
        self
    }
}

pub fn create_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_buffers: &[wgpu::VertexBufferLayout],
    shader_module_data: &str,

    desc: RenderPipelineDescriptor,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{} layout", label)),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{} shader module", label)),
        source: wgpu::ShaderSource::Wgsl(shader_module_data.into()),
    });

    let default_fragment_targets = [Some(wgpu::ColorTargetState {
        format: config.format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::all(),
    })];
    let fragment_targets = desc.fragment_targets.unwrap_or(&default_fragment_targets);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: vertex_buffers,
        },
        primitive: desc.primitive,
        depth_stencil: desc.depth_stencil,
        multisample: desc.multisample,
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: fragment_targets,
        }),
        multiview: desc.multiview,
        cache: desc.cache,
    })
}

//====================================================================

/// bind group layout uniform entry
pub fn bgl_uniform_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn bgl_storage_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn bgl_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub fn bgl_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

pub enum BufferType {
    Vertex,
    Index,
    Instance,
    Uniform,
}

pub fn buffer<D: bytemuck::Pod>(
    device: &wgpu::Device,
    buffer_type: BufferType,
    label: &str,
    data: &[D],
) -> wgpu::Buffer {
    let (name, usage) = match buffer_type {
        BufferType::Vertex => ("Vertex", wgpu::BufferUsages::VERTEX),
        BufferType::Index => ("Index", wgpu::BufferUsages::INDEX),
        BufferType::Instance => (
            "Instance",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        ),
        BufferType::Uniform => (
            "Uniform",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        ),
    };

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} {} Buffer", label, name)),
        contents: bytemuck::cast_slice(data),
        usage,
    })
}

//====================================================================

pub fn update_instance_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,

    label: &str,
    buffer: &mut wgpu::Buffer,
    instance_count: &mut u32,

    data: &[T],
) {
    if data.len() == 0 {
        // Nothing to update
        if *instance_count != 0 {
            // Empty buffer and reset instance count
            *buffer = create_instance_buffer(device, label, data);
            *instance_count = 0;
        }

        return;
    }

    // We can fit all data inside existing buffer
    if data.len() <= *instance_count as usize {
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
        *instance_count = data.len() as u32; // TODO - add additional variable for buffer size
        return;
    }

    // Buffer is too small to fit new data. Create a new bigger one.
    *instance_count = data.len() as u32;
    *buffer = create_instance_buffer(device, label, data);
}

pub fn create_instance_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Instance Buffer", label)),
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}

//====================================================================

pub struct InstanceBuffer<T> {
    phantom: PhantomData<T>,
    buffer: wgpu::Buffer,
    count: u32,
}

impl<T: bytemuck::Pod> InstanceBuffer<T> {
    #[inline]
    pub fn new(device: &wgpu::Device, data: &[T]) -> Self {
        Self {
            phantom: PhantomData,
            buffer: buffer(
                device,
                BufferType::Instance,
                &format!("{} Instance Buffer", std::any::type_name::<T>()),
                data,
            ),
            count: data.len() as u32,
        }
    }

    #[inline]
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        update_instance_buffer(
            device,
            queue,
            &format!("{} Instance Buffer", std::any::type_name::<T>()),
            // "Instance Buffer",
            &mut self.buffer,
            &mut self.count,
            data,
        );
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    #[inline]
    pub fn count(&self) -> u32 {
        self.count
    }
}

//====================================================================

// pub fn calculate_model_normals(vertices: &mut [ModelVertex], indices: &[u16]) {
//     let mut vertex_acc = vec![(0, glam::Vec3::ZERO); vertices.len()];

//     let triangle_count = indices.len() / 3;

//     (0..triangle_count).for_each(|index| {
//         let index = index * 3;

//         let i1 = indices[index] as usize;
//         let i2 = indices[index + 1] as usize;
//         let i3 = indices[index + 2] as usize;

//         let v1: glam::Vec3 = vertices[i1].position.into();
//         let v2: glam::Vec3 = vertices[i2].position.into();
//         let v3: glam::Vec3 = vertices[i3].position.into();

//         let u = v2 - v1;
//         let v = v3 - v1;

//         // let normal = u.cross(v);
//         let normal = v.cross(u);

//         vertex_acc[i1].0 += 1;
//         vertex_acc[i1].1 += normal;

//         vertex_acc[i2].0 += 1;
//         vertex_acc[i2].1 += normal;

//         vertex_acc[i3].0 += 1;
//         vertex_acc[i3].1 += normal;
//     });

//     vertex_acc
//         .into_iter()
//         .enumerate()
//         .for_each(|(index, (count, normal))| {
//             if count == 0 {
//                 log::warn!(
//                     "Calculate model normals: Vertex {} not used in any triangles",
//                     index
//                 );
//                 return;
//             }

//             let normal = normal.try_normalize().unwrap_or(glam::Vec3::ZERO);
//             vertices[index].normal = normal.to_array();
//         });
// }

//====================================================================
