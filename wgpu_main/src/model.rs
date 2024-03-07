use crate::texture::Texture;
use std::ops::Range;
use wgpu::BindGroup;

/// Making `Vertex` a trait will allow us to abstract out the `VertexBufferLayout` creation code to
/// make creating `RenderPipeline`s easier.
pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

/// `bytemuck::Pod` indicates that our `Vertex` is "Plain Old Data" and can be interpreted as a &[u8].
/// `bytemuck::Zeroable` indicates that we can use `std::mem::zeroed()`.
///
/// # Tangent Space to World Space
/// When we pull the normal data from our normal texture, all the normals are in what's known as pointing
/// roughly in the positive z direction. That means that our lighting calculation thinks all the
/// surfaces of our models are facing roughly in the same direction. This is referred to as `tangent space`.
/// It turns out that we can use the vertex normal (indicating the direction of the surface), to
/// transform our normals from `tangent space` into `world space`.
///
/// We can create a matrix that represents a coordinate system using three vectors that are perpendicular
/// (or orthonormal) to each other. We're going to create a matrix that will represent the coordinate
/// space relative to our vertex normals. We're then going to use that to transform our normal map
/// data to be in world space.
///
/// # The tangent and the bitangent
///
/// A tangent represents any vector parallel with a surface (aka. doesn't intersect the surface). The
/// tangent is always perpendicular to the normal vector. The bitangent is a tangent vector that is
/// perpendicular to the other tangent vector. Together, the tangent, bitangent and normal represent
/// the x, y, and z axes of a coordinate system.
///
/// Some model formats include the tangent and bitangent (sometimes called the binormal) in the vertex
/// data, but OJB does not. We'll have to calculate them manually. Luckily, we can derive our tangent
/// and bitangent from our existing vertex data.
/// ![Tangent and Bitangent][normal_tangent_bitangent_coordinate_system.png]
///
/// Basically, we can use the edges of our triangles and our normal to calculate the tangent and bitangent.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
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
                // Tangent and bitangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
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
    // The r, g and b components of the texture correspond to the x, y and z components of the normal.
    // All z values should be positive. That's why the normal map has a bluish tint.
    pub normal_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: Texture,
        normal_texture: Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: None,
        });
        Self {
            name: name.to_string(),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
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
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
        instances: Range<u32>,
    );

    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    );

    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, camera_bind_group, light_bind_group, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'a Material,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        // When using an index buffer, you need to use draw_indexed. The draw method ignores the
        // index buffer. Also, make sure you use the number of indices, not vertices, as your
        // model will either draw wrong or the method will panic because there are not enough indices.
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b BindGroup,
    ) {
        self.draw_model_instanced(model, camera_bind_group, light_bind_group, 0..1);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b BindGroup,
        instances: Range<u32>,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(
                mesh,
                material,
                camera_bind_group,
                light_bind_group,
                instances.clone(),
            );
        }
    }
}
pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}
