use crate::camera::Camera;
use cgmath::InnerSpace;
use winit::event::KeyEvent;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey::Code};
pub(crate) struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub(crate) fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub(crate) fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match physical_key {
                    Code(KeyCode::KeyW) | Code(KeyCode::ArrowUp) => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    Code(KeyCode::KeyS) | Code(KeyCode::ArrowDown) => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    Code(KeyCode::KeyA) | Code(KeyCode::ArrowLeft) => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    Code(KeyCode::KeyD) | Code(KeyCode::ArrowRight) => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = (camera.target - camera.eye);
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the center of the scene.
        if self.is_right_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so that it doesn't change. The
            // eye, therefore, still lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }

        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
