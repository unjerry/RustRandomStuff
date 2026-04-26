#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub origin: [f32; 4],
    pub look_at: [f32; 4],
    pub params: [f32; 4],
}

impl CameraUniform {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            origin: [0.0, 4.0, 4.0, 1.0],
            look_at: [0.0, 0.0, 0.0, 1.0],
            params: [
                width as f32 / height as f32,
                45.0_f32.to_radians(),
                0.0,
                0.0,
            ],
        }
    }
}
