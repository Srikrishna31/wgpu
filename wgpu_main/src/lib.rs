mod state;

use state::State;
use winit::{
    error::EventLoopError,
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

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == state.window().id() => match event {
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
                _ => (),
            },
            _ => (),
        })
        .expect("TODO: panic message");
}