use crate::model::{Material, Mesh, Model, ModelVertex};
use crate::texture::Texture;
use cfg_if::cfg_if;
/// By design, you can't access files on a user's filesystem in Web Assembly. Instead, we'll serve
/// those files up using a web serve and then load those files into our code using an http request.
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Device, Queue};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = {
        let origin = location.origin().unwrap();
        if !origin.ends_with("learn-wgpu") {
            format!("{}/learn-wgpu", origin)
        } else {
            origin
        }
    };
    let base = reqwest::Url::parse(&format!("{origin}/")).unwrap();
    base.join(file_name).unwrap()
}

pub(crate) async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url).await?.text().await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("models").join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub(crate) async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let bytes = reqwest::get(url).await?.bytes().await?.to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("models").join(file_name);
            let bytes = std::fs::read(path)?;
        }
    }

    Ok(bytes)
}

pub(crate) async fn load_model(
    file_name: &str,
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture =
            Texture::load_texture(&m.diffuse_texture.unwrap(), device, queue).await?;
        let normal_texture =
            Texture::load_texture(&m.normal_texture.unwrap(), device, queue).await?;

        materials.push(Material::new(
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let meshes = models
        .iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: if m.mesh.normals.is_empty() {
                        [0.0; 3]
                    } else {
                        [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ]
                    },
                })
                .collect::<Vec<_>>();
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{file_name} vertex buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{file_name} index buffer")),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            Mesh {
                name: m.name.clone(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect();

    Ok(Model { meshes, materials })
}
