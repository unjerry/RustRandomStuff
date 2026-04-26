// 1. 定义 Vertex 结构体
#[repr(C)] // 保证 C 语言风格的内存布局，防止 Rust 编译器重排字段
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // bytemuck 魔法
pub struct Vertex {
    pub position: [f32; 3], // x, y, z
    pub color: [f32; 3],    // r, g, b
}
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 一个顶点总共占多宽
            step_mode: wgpu::VertexStepMode::Vertex, // 每个顶点更新一次数据
            attributes: &[
                // 位置信息 (location 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // 颜色信息 (location 1)
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // 跳过位置占据的字节
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
