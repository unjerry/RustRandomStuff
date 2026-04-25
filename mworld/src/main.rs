use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
// 我们去掉生命周期 'a，让 State 稍微简单一点
struct State {
    surface: wgpu::Surface<'static>, // 使用 'static 是因为我们会确保 Surface 不会比窗口活得久
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // window: Window, // 如果需要，也可以把 window 存进来
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

        Self {
            surface,
            device,
            queue,
            config,
            size,
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
            // 这里必须写出所有字段，因为 RenderPassDescriptor 不支持 Default
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                    // 修复 E0063: 手动指定深度切片，通常 2D 渲染设为 None
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                // 修复 E0063: 手动指定多视图掩码，通常设为 None
                multiview_mask: None,
            });
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
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
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
