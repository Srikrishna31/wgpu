use crate::texture::Texture;
use wgpu::Operations;

/// # High Dynamic Range Rendering
/// When we use `TextureFormat::Bgra8UnormSrgb` for the surface texture, it means that we have 8 bits
/// for each red, green, blue and alpha channel. While the channels are stored as integers between 0
/// and 255 inclusively, they get converted to and from floating point values between 0.0 and 1.0.
///
/// The problem with this is, most of the precision gets used to represent darker values of the scene.
/// This means that bright objects like light bulbs have the same value as exceedingly bright objects
/// like the sun. This inaccuracy makes realistic lighting difficult to do right. Because of this, we
/// are going to switch our rendering sytesm to use high dynamic range (HDR) in order to give our
/// scene more flexibility and enable us to leverage more advanced techniques such as Physically Based
/// Rendering.
///
/// ## High Dynamic Range
/// A High Dynamic Range texture is a texture with more bits per pixel. In addition to this, HDR
/// textures are stored as floating point values instead of integer values. This means that the
/// texture can have brightness values greater than 1.0, meaning you can have a dynamic range of
/// brighter objects.
///
/// ## Switching to HDR
/// Currently wgpu doesn't allow us to use a floating point format such as `TextureFormat::Rgba16Float`
/// as the surface texture format (not all monitors support that anyway), so we will have to render
/// our scene in an HDR format, then convert the values to a supported format such as
/// `TextureFormat::Bgra8UnormSrgb` before displaying them on the screen, using a technique called
/// tone mapping.
pub(crate) struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
}

impl HdrPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let width = config.width;
        let height = config.height;

        // We could use `Rgba32Float`, but that requires some extra features to be enabled for
        // rendering.
        let format = wgpu::TextureFormat::Rgba16Float;

        let texture = Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("hdr_texture"),
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hdr_bind_group_layout"),
            entries: &[
                // This is the HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hdr_bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let shader = wgpu::include_wgsl!("shaders/hdr.wgsl");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = crate::State::create_render_pipeline(
            device,
            &pipeline_layout,
            config.format,
            None,
            // We'll use some math to generate the vertex data in the shader, so we don't need any
            // vertex buffers
            &[],
            shader,
            wgpu::PrimitiveTopology::TriangleList,
        );

        Self {
            pipeline,
            bind_group,
            texture,
            width,
            height,
            format,
            layout,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = Texture::create_2d_texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("hdr_texture"),
        );
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hdr_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
            ],
        });
        self.width = width;
        self.height = height;
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// This renders the internal HDR texture to the [TextureView] supplied as parameter.
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hdr_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
