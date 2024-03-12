use crate::resources::load_binary;
use anyhow::*;
use image::GenericImageView;
use wgpu::{Device, Queue, SurfaceConfiguration};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label), is_normal_map)
    }

    /// # Srgb and normal textures
    /// We've been using `Rgba8UnormSrgb` for all of our textures. The `Srgb` bit specifies that we
    /// will be using standard RGB color space. This is also known as linear color space. Linear color
    /// space has less color density. Even so, it is often used for diffuse textures, as they are
    /// typically made in `Srgb` color space.
    ///
    /// Normal textures aren't made with `Srgb`. Using `Rgba8UnormSrgb` can change how the GPU samples
    /// the texture. This can make the resulting simulation less accurate. We can avoid these issues
    /// by using `Rgba8Unorm` for normal textures.
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            // Most images are stored using sRGB, so we need to reflect that here.
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            // All textures are stored as 3D, we represent our 2D texture by setting depth to 1.
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders,
            // COPY_DST means that we want to copy data to this texture.
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            // This is the same as with the SurfaceConfig. It specifies what texture formats can be
            // used to create TextureViews for this texture. The base texture format (Rgba8UnormSrgb
            // in this case) is always supported. Note that using a different texture format is not
            // supported on the WebGL2 backend.
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Ok(Self {
            texture,
            view,
            sampler,
            size,
        })
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// We need the `DEPTH_FORMAT` for creating the depth stage of the `render_pipeline` and for
    /// creating the depth texture itself.
    pub fn create_depth_texture(
        device: &Device,
        config: &SurfaceConfiguration,
        label: &str,
    ) -> Texture {
        // Our depth texture needs to be the same size as our screen if we want things to render correctly.
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            // Since we are rendering to this texture, we need to add the RENDER_ATTACHMENT usage.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // We technically don't need a sampler for the depth texture, but out `Texture` struct requires
        // it, and we need one if we ever want to sample it.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            // If we do decide to render our depth texture, we need to use `CompareFunction::LessEqual`.
            // This is due to how the `sampler_comparison` and `textureSampleCompare()` interact with
            // the `texture()` function in GLSL.
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_max_clamp: 100.0,
            lod_min_clamp: 0.0,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            size,
        }
    }

    /// The `load_texture` method will be useful when we load the textures for our models, as
    /// `include_bytes!` requires that we know the name of the file at compile time, which we can't
    /// really guarantee with model textures.
    pub async fn load_texture(
        file_name: &str,
        device: &Device,
        queue: &Queue,
        is_normal_map: bool,
    ) -> Result<Texture> {
        let data = load_binary(file_name).await?;

        Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
    }

    pub fn create_2d_texture(
        device: &Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        Self::create_texture(
            device,
            label,
            size,
            format,
            usage,
            wgpu::TextureDimension::D2,
            mag_filter,
        )
    }

    fn create_texture(
        device: &wgpu::Device,
        label: Option<&str>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        dimension: wgpu::TextureDimension,
        mag_filter: wgpu::FilterMode,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
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
            view,
            sampler,
            size,
        }
    }
}
