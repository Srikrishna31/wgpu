use cgmath::prelude::*;
use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use wgpu::Device;

/// Instancing allows us to draw the same object multiple times with different properties (position,
/// orientation, size, color, etc.). There are multiple ways of doing instancing. One way would be to
/// modify the uniform buffer to include these properties and then update it before we draw each
/// instance of our object.
/// We don't want to use this method for performance reasons. Updating the uniform buffer for each
/// instance would require multiple buffer copies for each frame. On top of that, our method to update
/// the uniform buffer currently requires us to create a new buffer to store the updated data.
///
/// A `Quaternion` is a mathematical structure often used to represent rotation. Using these values
/// directly in the shader would be a pain, as quaternions don't have a WGSL analog. So, we'll convert
/// the `Instance` data into a matrix and store it in a struct called `InstanceRaw`.
pub(crate) struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

/// This is the data that goes into wgpu::Buffer. We keep these separate so that we can update `Instance`
/// as much as we want without needing to mess with matrices. We only need to update the raw data
/// before we draw.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl Instance {
    pub(crate) fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
        }
    }

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    pub(crate) fn create_instances(device: &Device) -> (Vec<Instance>, wgpu::Buffer) {
        let instances = (0..Self::NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..Self::NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - Self::INSTANCE_DISPLACEMENT;
                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) doesn't get scaled to zero
                        // as Quaternions can't affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };
                    Self { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        (instances, instance_buffer)
    }
}

impl InstanceRaw {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance. This means that our
            // shaders will only change to use the next instance when the shader starts processing
            // a new instance.
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a
                // slot for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
