use crate::parts::state::State;
use std::sync::Arc;
use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};
#[derive(Default)]
pub struct App {
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
