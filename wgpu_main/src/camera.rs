pub(crate) struct Camera {
    pub(crate) eye: cgmath::Point3<f32>,
    pub(crate) target: cgmath::Point3<f32>,
    pub(crate) up: cgmath::Vector3<f32>,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

/// The coordinate system in Wgpu is based on DirectX and Metal's coordinate systems. That means that
/// in normalized device coordinates, the x-axis and y-axis are in the range of -1 to 1, and the z-axis
/// is 0.0 to +1.0. The `cgmath` crate (as well as most game math crates) is built for OpenGL's coordinate
/// system. This matrix will scale and translate our scene from OpenGL's coordinate system to Wgpu's.
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.5, 0.5,
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // The `view` matrix moves the world to be at the position and rotation of the camera. It's
        // essentially an inverse of whatever the transform matrix of the camera would be.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // The `proj` matrix wraps the scene to give the effect of depth. Without this, objects up
        // close would look the same as objects far away.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
