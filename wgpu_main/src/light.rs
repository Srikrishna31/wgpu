use crate::texture::Texture;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferBindingType,
    Device,
};

/// In the real world, a light source emits photons that bounce around until they enter our eyes.
/// The color we see is the light's original color minus whatever energy it lost while bouncing around.
///
/// #Ray Tracing
/// It's the closest model to the way light really works.
///
/// # The Blinn-Phong Model
/// Ray/path tracing is often too computationally expensive for most real-time applications(though
/// that is starting to change), so a more efficient, if less accurate method based on the
/// `Phong reflection model` is often used. It splits up the lighting calculation into three parts:
/// ambient lighting, diffuse lighting, and specular lighting. The `Blinn-Phong model` is a modification
/// of the `Phong reflection model`, which cheats a bit at the specular calculation to speed things up.
///
/// The `LightUniform` represents a colored point in space. It is used to represent the light source
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub(crate) position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub(crate) color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        LightUniform {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        }
    }

    pub fn create_bind_group(device: &Device) -> (Buffer, BindGroupLayout, BindGroup) {
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[LightUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        (light_buffer, light_bind_group_layout, light_bind_group)
    }
}

impl Default for LightUniform {
    fn default() -> LightUniform {
        LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        }
    }
}
