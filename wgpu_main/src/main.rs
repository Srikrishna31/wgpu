use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey::Code},
    error::EventLoopError,
};

fn run() -> Result<(), EventLoopError> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event,elwt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
            ..
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event: KeyEvent {
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
}
fn main() {
    run().unwrap();
}
