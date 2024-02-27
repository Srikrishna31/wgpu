mod state;
mod texture;

use state::State;
use winit::{
    error::EventLoopError,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey::Code},
    window::WindowBuilder,
};

/// `bytemuck::Pod` indicates that our `Vertex` is "Plain Old Data" and can be interpreted as a &[u8].
/// `bytemuck::Zeroable` indicates that we can use `std::mem::zeroed()`.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // The ```array_stride``` defines how wide a vertex is. When the shader goes to read the
            // next vertex, it will skip over this many bytes to get to the next vertex.
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                    // Format thells the shader the shape of the attribute. Float32x3 corresponds to vec3<f32>
                    // in shader code.
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// pub(crate) const VERTICES: &[Vertex] = &[
//     Vertex {
//         position: [0.0, 0.5, 0.0],
//         color: [1.0, 0.0, 0.0],
//     }, // A
//     Vertex {
//         position: [-0.5, -0.5, 0.0],
//         color: [0.0, 1.0, 0.0],
//     }, // B
//     Vertex {
//         position: [0.5, -0.5, 0.0],
//         color: [0.0, 0.0, 1.0],
//     }, // C
// ];

pub(crate) const PENTAGON: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 1.0 - 0.99240386],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 1.0 - 0.56958647],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 1.0 - 0.05060294],
    }, // C
    Vertex {
        position: [0.35966998, -0.3743291, 0.0],
        tex_coords: [0.85967, 1.0 - 0.1526709],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 1.0 - 0.7347359],
    }, // E
];

pub(crate) const INDICES: &[u16] = &[
    0, 1, 4, // ABD
    1, 2, 4, // BCE
    2, 3, 4, // CDE
];

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could not initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to do it manually on the web
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(800, 600));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example").unwrap();
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                match dst.append_child(&canvas) {
                    Ok(_) => Some(()),
                    Err(e) => {
                        eprintln!("Couldn't append canvas to document body: {:?}", e);
                        None
                    }
                }
            })
            .expect("Couldn't append canvas to document body");
    }

    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: factor,
                            ..
                        } => {
                            let new_size = {
                                let size = state.window().inner_size();
                                winit::dpi::PhysicalSize::new(
                                    (size.width as f64 * factor) as u32,
                                    (size.height as f64 * factor) as u32,
                                )
                            };
                            state.resize(new_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => (),
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.window().inner_size())
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        })
        .expect("TODO: panic message");
}
