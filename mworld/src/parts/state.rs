use crate::parts::{
    camera::CameraUniform,
    pipeline::{create_camera_bind_group_layout, create_render_pipeline},
    vertex::Vertex,
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

const SHADER_SOURCE: &str = include_str!("shader.wgsl");
// const SHADER_SOURCE: &str = "";
// 2. 定义四个顶点数据（逆时针顺序：左上，左下，右下，右上）
const VERTICES: &[Vertex] = &[
    // NDC 坐标 (-1, 1, -1, 1)，留边 0.9
    Vertex {
        position: [-0.9, 0.9, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // 0: 左上 (红)
    Vertex {
        position: [-0.9, -0.9, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // 1: 左下 (绿)
    Vertex {
        position: [0.9, -0.9, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // 2: 右下 (蓝)
    Vertex {
        position: [0.9, 0.9, 0.0],
        color: [1.0, 1.0, 0.0],
    }, // 3: 右上 (黄)
];

// 3. 定义索引，组成两个逆时针三角形
const INDICES: &[u16] = &[
    0, 1, 2, // 三角形 1
    0, 2, 3, // 三角形 2
];
pub struct State {
    surface: wgpu::Surface<'static>, // 使用 'static 是因为我们会确保 Surface 不会比窗口活得久
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // window: Window, // 如果需要，也可以把 window 存进来
    // --- 新增字段 ---
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32, // 记录有多少个索引需要画
    render_pipeline: wgpu::RenderPipeline,
    // --- 新增 ---
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}
impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();

        // 关键点：传入 window (它实现了 SurfaceTarget 且是 'static)
        let surface = instance.create_surface(window).expect("创建surface失败");

        // 这里就是你之前卡住的地方：请求适配器 (显卡)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // 你能猜到下一步我们需要用这个 adapter 做什么吗？
        // 提示：我们需要从这块“物理显卡”上申请一个“逻辑连接”和“命令队列”。
        // 在 State::new 中接在获取 adapter 之后：
        // 1. 修改后的 request_device 调用
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    // 使用 ..Default::default() 可以自动填充你没写出来的字段
                    // 比如报错提到的 experimental_features 和 trace
                    ..Default::default()
                },
                // 这里删掉之前的 None 参数，因为它现在只收 1 个参数了
            )
            .await
            .expect("无法获取 Device 和 Queue");

        // 获取 Surface 的默认配置
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb()) // 优先选择 sRGB 格式
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // 我们要往这张“画纸”上渲染
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0], // 垂直同步等设置
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // 告诉 Surface 应用这个配置
        surface.configure(&device, &config);

        // --- 新增：创建 Vertex Buffer ---
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES), // 将 Rust 数组转换为字节切片
            usage: wgpu::BufferUsages::VERTEX,        // 告诉 GPU 这是顶点数据
        });

        // --- 新增：创建 Index Buffer ---
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX, // 告诉 GPU 这是索引数据
        });
        let num_indices = INDICES.len() as u32;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        // 1. 初始化相机数据
        let camera_uniform = CameraUniform::new(config.width, config.height);
        // 2. 创建 Camera Buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = create_camera_bind_group_layout(&device);

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline =
            create_render_pipeline(&device, &config, &shader, &camera_bind_group_layout);
        Self {
            surface,
            device,
            queue,
            config,
            size,
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
        }
    }
    // 1. 处理窗口缩放
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            // 关键：重新配置 Surface
            self.surface.configure(&self.device, &self.config);
            let aspect = self.config.width as f32 / self.config.height as f32;

            let camera_uniform = CameraUniform::new(self.config.width, self.config.height);

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );
        }
    } // 2. 渲染函数
    // 我们先用一个通用的 Result，避免找不到 SurfaceError 的问题
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 使用 match 处理枚举的不同情况
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => output, // 成功：拿到了 SurfaceTexture
            wgpu::CurrentSurfaceTexture::Outdated => {
                // 窗口过时了（通常发生在调整大小时），我们需要重新配置
                self.resize(self.size);
                return Ok(());
            }
            // 处理其他异常情况
            e => return Err(format!("Surface 错误: {:?}", e).into()),
        };

        // 现在 output 是 SurfaceTexture 类型了，它拥有 texture 字段！
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // 1. 开始录制，并把“句柄”存入 render_pass 变量
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, // 之前创建的视图
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                // ... 其他字段 (别忘了补上 multiview_mask: None)
                ..Default::default() // 或者手动补齐
            });

            // 1. 设置管线
            render_pass.set_pipeline(&self.render_pipeline);

            // 2. 绑定相机（戴上眼镜）
            // 对应 Shader 里的 @group(0) @binding(0)
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // 3. 绑定顶点和索引（准备画布）
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // 4. 下达指令（只画一次就够了）
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
} // 我们定义一个结构体来保存窗口和 wgpu 的状态
