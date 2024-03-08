use super::{camera::Camera, camera::SAFE_FRAC_PI_2};
use cgmath::{InnerSpace, Rad, Vector3};
use instant::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, MouseScrollDelta};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey::Code},
};

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    mouse_pressed: bool,
}

impl CameraController {
    pub(crate) fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            mouse_pressed: false,
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
                let amount = if *state == ElementState::Pressed {
                    1.0
                } else {
                    0.0
                };
                match physical_key {
                    Code(KeyCode::KeyW) | Code(KeyCode::ArrowUp) => {
                        self.amount_forward = amount;
                        true
                    }
                    Code(KeyCode::KeyS) | Code(KeyCode::ArrowDown) => {
                        self.amount_backward = amount;
                        true
                    }
                    Code(KeyCode::KeyA) | Code(KeyCode::ArrowLeft) => {
                        self.amount_left = amount;
                        true
                    }
                    Code(KeyCode::KeyD) | Code(KeyCode::ArrowRight) => {
                        self.amount_right = amount;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.mouse_pressed {
            self.rotate_horizontal = mouse_dx as f32 * self.sensitivity;
            self.rotate_vertical = mouse_dy as f32 * self.sensitivity;
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // Assuming a line is about 100 pixels.
            MouseScrollDelta::LineDelta(_, y) => *y * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => *y as f32,
        };
    }

    pub(crate) fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position changes when zooming. This is added
        // to make it easier to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += cgmath::Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += cgmath::Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values will not get set to zero, and the
        // camera will rotate when moving in a non-cardinal direction
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2)
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2)
        }
    }
}
