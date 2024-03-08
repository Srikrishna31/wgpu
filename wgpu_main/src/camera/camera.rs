use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};
use std::f32::consts::FRAC_PI_2;

/// A camera that can be moved and rotated, in FPS style - so we'll store the position and the yaw
/// (horizontal rotation), and pitch (vertical rotation).
pub struct Camera {
    pub position: Point3<f32>,
    pub(super) yaw: Rad<f32>,   // Represents Horizontal rotation
    pub(super) pitch: Rad<f32>, // Represents Vertical Rotation
}

pub(super) const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

/// The coordinate system in Wgpu is based on DirectX and Metal's coordinate systems. That means that
/// in normalized device coordinates, the x-axis and y-axis are in the range of -1 to 1, and the z-axis
/// is 0.0 to +1.0. The `cgmath` crate (as well as most game math crates) is built for OpenGL's coordinate
/// system. This matrix will scale and translate our scene from OpenGL's coordinate system to Wgpu's.
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    /// This creates the view matrix.
    fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
    // fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    //     // The `view` matrix moves the world to be at the position and rotation of the camera. It's
    //     // essentially an inverse of whatever the transform matrix of the camera would be.
    //     let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
    //     // The `proj` matrix wraps the scene to give the effect of depth. Without this, objects up
    //     // close would look the same as objects far away.
    //     let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    //
    //     OPENGL_TO_WGPU_MATRIX * proj * view
    // }
}

/// The projection only needs to change if the window resizes, so we'll store it separately.
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
/// A uniform is a blob of data available to every invocation of a set of shaders.
// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    // We can't use cgmath with bytemuck directly, so we'll convert the Matrix4 into a 4x4 f32 array.
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        // We're using Vector4 because of the uniforms 16byte alignment requirement
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}
