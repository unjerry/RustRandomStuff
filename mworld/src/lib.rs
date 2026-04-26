mod parts;
use crate::parts::app::App;
use winit::event_loop::{ControlFlow, EventLoop};
pub fn run() {
    env_logger::init(); // 初始化日志，方便调试 wgpu
    let event_loop = EventLoop::new().expect("创建eventloop失败");
    // 设置循环模式为持续运行
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    // 启动程序
    let _ = event_loop.run_app(&mut app);
}
