use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
// 1. 定义 Vertex 结构体
#[repr(C)] // 保证 C 语言风格的内存布局，防止 Rust 编译器重排字段
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // bytemuck 魔法
struct Vertex {
    position: [f32; 3], // x, y, z
    color: [f32; 3],    // r, g, b
}
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
const SHADER_SOURCE: &str = "
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    // 目前我们直接输出位置，之后相机矩阵会在这里发挥作用
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
";
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
struct State {
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
}
impl State {
    async fn new(window: Arc<Window>) -> Self {
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
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[], // 目前还没有外部资源
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 对应 WGSL 里的 @vertex 函数名
                buffers: &[Vertex::desc()],   // 传入顶点布局说明书
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"), // 对应 WGSL 里的 @fragment 函数名
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // 直接覆盖背景色
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 我们要画三角形
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,  // 逆时针为正面
                cull_mode: Some(wgpu::Face::Back), // 剔除背面
                ..Default::default()
            },
            depth_stencil: None, // 目前不使用深度测试
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            surface,
            device,
            queue,
            config,
            size,
            // --- 填充新增字段 ---
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
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

            // 2. 既然我们手里有了 render_pass 这支笔，就可以开始画画了
            render_pass.set_pipeline(&self.render_pipeline); // 设置刚才创建的管线
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..)); // 绑定顶点数据
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 绑定索引

            // 3. 下达最后的开火命令！
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // 注意：当 render_pass 离开这个大括号作用域时，录制就自动结束了            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
} // 我们定义一个结构体来保存窗口和 wgpu 的状态
#[derive(Default)]
struct App {
    // 将 Window 包装在 Arc 中
    window: Option<Arc<Window>>,
    state: Option<State>,
} // 为 App 实现 winit 的核心处理器
impl ApplicationHandler for App {
    // 当程序启动或从后台恢复时触发（这是创建窗口的最佳时机）
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes().with_title("WGPU 窗口 🚀");
            // 1. 创建并包装窗口
            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("创建window失败"),
            );
            self.window = Some(window.clone());
            // 2. 初始化 State (使用 pollster 同步等待异步结果)
            let state = pollster::block_on(State::new(window));
            self.state = Some(state);
            println!("窗口与 WGPU State 初始化成功！");
        }
    } // 处理各种窗口事件，比如关闭、按键等
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // 窗口缩放时，告诉 state 更新配置
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(new_size);
                }
            }
            // 关键：当系统要求重绘时调用我们的 render
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    if let Err(e) = state.render() {
                        eprintln!("渲染出错: {:?}", e);
                    }
                }
                // 持续请求重绘，让画面动起来
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}
fn main() {
    env_logger::init(); // 初始化日志，方便调试 wgpu
    let event_loop = EventLoop::new().expect("创建eventloop失败");
    // 设置循环模式为持续运行
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    // 启动程序
    let _ = event_loop.run_app(&mut app);
}
