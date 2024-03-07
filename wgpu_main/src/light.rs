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
///
/// # Ambient Lighting
/// Light has a tendency to bounce around and fill in the shadows. This is called `ambient lighting`.
/// Modeling this interaction would be computationally expensive, so we just fake it by adding a small
/// ambient lighting value for the light bouncing off other parts of the scene to light our objects.
/// The ambient part is based on the light color and the object color.
///
/// # Diffuse Lighting
/// Normals represent the direction a surface is facing. By comparing the normal of a fragment with a
/// vector pointint to a light source, we get a value of how light/dark that fragment should be. We
/// compare the vectors using the dot product to get the cosine of the angle between them.
/// ![Diffuse Lighting][normal_diagram.png]
///
/// If the dot product of the normal and light vector is 1.0, that means that the current fragment is
/// directly in line with the light source, and will receive the light's full intensity. A value of
/// 0.0 or lower means that the fragment is perpendicular or facing away from the light source, and
/// therefor will be dark.
///
/// # Specular Lighting
/// It describes the highlights that appear on objects when viewed from certain angles.
/// Basically, some of the light can reflect off the surface like a mirror. The location of the highlight
/// shifts depending on what angle you view it at.
/// ![Specular Lighting][specular_diagram.png]
/// Because this is relative to the view angle, we are going to need to pass in the camera's position
/// both into the fragment shader and into the vertex shader.
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
