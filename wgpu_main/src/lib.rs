mod camera;
mod instance;
mod light;
mod model;
mod resources;
mod state;
mod texture;

use state::State;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey::Code},
    window::WindowBuilder,
};

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
    let mut last_render_time = instant::Instant::now();

    event_loop
        .run(move |event, elwt| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1);
            },
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        #[cfg(not(target_arch = "wasm32"))]
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
                            let now = instant::Instant::now();
                            let dt = now - last_render_time;
                            last_render_time = now;
                            state.update(dt);
                            match state.render() {
                                Ok(_) => (),
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.window().inner_size())
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                        }

                        _ => (),
                    }
                }
            }
            // MainEventsCleared has been renamed to AboutToWait as per the information here:
            // https://github.com/rust-windowing/winit/issues/2900
            Event::AboutToWait => {
                state.window().request_redraw();
            }
            _ => (),
        })
        .expect("TODO: panic message");
}
