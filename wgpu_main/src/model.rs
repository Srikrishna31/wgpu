/// Making `Vertex` a trait will allow us to abstract out the `VertexBufferLayout` creation code to
/// make creating `RenderPipeline`s easier.
pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

/// `bytemuck::Pod` indicates that our `Vertex` is "Plain Old Data" and can be interpreted as a &[u8].
/// `bytemuck::Zeroable` indicates that we can use `std::mem::zeroed()`.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            // The ```array_stride``` defines how wide a vertex is. When the shader goes to read the
            // next vertex, it will skip over this many bytes to get to the next vertex.
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            // step_mode tells the pipeline whether each element of the array in this buffer represents
            // pre-vertex data or per-instance data.
            step_mode: wgpu::VertexStepMode::Vertex,
            // Vertex attributes describe the individual parts of a vertex.
            attributes: &[
                wgpu::VertexAttribute {
                    // This defines the offset in bytes until the attribute starts. For the first attribute,
                    // the offset is usually zero. For any latter attributes, the offset is the sum over
                    // size_of of the previous attribute's data.
                    offset: 0,
                    // This tells the shader what location to store this attribute at. FOr example,
                    // `@location(0) x: vec3<f32>` in the vertex shader would correspond to the position
                    // field of the Vertex struct, while `@location(1) x: vec3<f32>` would be the color field.
                    shader_location: 0,
                    // Format tells the shader the shape of the attribute. Float32x3 corresponds to vec3<f32>
                    // in shader code.
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
