use core::default::Default;

/// # Equirectangular textures
/// An equirectangular texture is a texture where a sphere is stretched across a rectangular surface
/// using what's known as an equirectangular projection. This map of the Earth is an example of
/// this projection:
/// ![Equirectangular Projection][Equirectangular_projection_SW.jpg]
///
/// This projection maps the latitude values of the spheres to the horizontal coordinates of the
/// texture. The longitude values get mapped to the vertical coordinates. This means that the vertical
/// middle of the texture is the equator (0° longitude), of the sphere, the horizontal middle is the
/// prime meridian (0° latitude) of the sphere, the left and right edges of the texture are the
/// anti-meridian(+180°/-180° latitude) of the sphere, and the top and bottom edges of the texture
/// are the North Pole(+90° longitude) and the South Pole (-90° longitude) respectively.
/// ![Equirectangular][equirectangular.svg]
///
/// This simple projection is easy to use, making it one of the most popular projections for storing
/// spherical textures. It is also the most common format for storing High Dynamic Range (HDR) images.
///
/// # Cube Maps
///
/// A cube map is a special kind of texture that has six layers. Each layer corresponds to a different
/// face of an imaginary cube that is aligned to the X, Y, and Z axes. The layers are stored in the
/// following order: +X, -X, +Y, -Y, +Z, -Z. This is the same order that the faces are stored in the
/// equirectangular texture.
pub(crate) struct CubeTexture {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    view: wgpu::TextureView,
}

impl CubeTexture {
    pub fn create_2d(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            sampler,
            view,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
