use crate::texture::Texture;
use std::ops::Range;

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

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

/// `Mesh` holds a vertex buffer, an index buffer, and the number of indices in the mesh. We're
/// using a `usize` for the material. This `usize` will index the `materials` list when it is time to draw.
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // When using an index buffer, you need to use draw_indexed. The draw method ignores the
        // index buffer. Also, make sure you use the number of indices, not vertices, as your
        // model will either draw wrong or the method will panic because there are not enough indices.
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}
