//====================================================================

use std::sync::{atomic::AtomicU32, Arc};

use super::{shared::SharedRenderResources, texture::Texture};

//====================================================================

static CURRENT_TEXTURE_ID: AtomicU32 = AtomicU32::new(0);

pub struct LoadedTexture {
    id: u32,
    _texture: Texture,
    bind_group: wgpu::BindGroup,
}

impl LoadedTexture {
    pub fn load_texture(
        device: &wgpu::Device,
        shared: &SharedRenderResources,
        texture: Texture,
    ) -> Self {
        let id = CURRENT_TEXTURE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let bind_group = shared.create_bind_group(device, &texture, None);
        Self {
            id,
            _texture: texture,
            bind_group,
        }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn _texture(&self) -> &Texture {
        &self._texture
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl PartialEq for LoadedTexture {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

//====================================================================

pub struct DefaultTexture {
    texture: Arc<LoadedTexture>,
}

impl DefaultTexture {
    #[inline]
    pub fn new(texture: Arc<LoadedTexture>) -> Self {
        Self { texture }
    }

    #[inline]
    pub fn get(&self) -> Arc<LoadedTexture> {
        self.texture.clone()
    }

    #[inline]
    pub fn texture(&self) -> &Arc<LoadedTexture> {
        &self.texture
    }
}

//====================================================================
