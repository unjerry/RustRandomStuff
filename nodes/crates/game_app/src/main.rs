use egui::{Context, ViewportId};
use egui_wgpu::{winit::Painter, WgpuConfiguration};
use egui_winit::State as EguiWinitState;
use nodes_core::NodeTemplate;
use nodes_ui::{show_node_editor, NodeGraphEditor, NodeRenderPlan};
use std::error::Error;
use std::fs;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

fn main() -> Result<(), Box<dyn Error>> {
    let template = load_template("assets/node_templates/ore_source.json")?;
    let event_loop = EventLoop::new()?;
    let mut app = NodeApp::new(template);

    event_loop.run_app(&mut app)?;
    Ok(())
}

fn load_template(path: &str) -> Result<NodeTemplate, Box<dyn Error>> {
    let source = fs::read_to_string(path)?;
    Ok(NodeTemplate::from_json_str(&source)?)
}

struct NodeApp {
    egui_ctx: Context,
    egui_state: Option<EguiWinitState>,
    painter: Option<Painter>,
    window: Option<Arc<Window>>,
    graph_editor: NodeGraphEditor,
}

impl NodeApp {
    fn new(template: NodeTemplate) -> Self {
        let render_plan = NodeRenderPlan::from_template(&template);
        Self {
            egui_ctx: Context::default(),
            egui_state: None,
            painter: None,
            window: None,
            graph_editor: NodeGraphEditor::new(render_plan),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let window_attributes = Window::default_attributes()
            .with_title("Nodes - JSON Node Prototype")
            .with_inner_size(LogicalSize::new(1040.0, 720.0));
        let window = Arc::new(event_loop.create_window(window_attributes)?);

        let mut egui_state = EguiWinitState::new(
            self.egui_ctx.clone(),
            ViewportId::ROOT,
            event_loop,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        let mut painter = pollster::block_on(Painter::new(
            self.egui_ctx.clone(),
            WgpuConfiguration::default(),
            1,
            None,
            false,
            false,
        ));
        pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone())))?;

        if let Some(max_texture_side) = painter.max_texture_side() {
            egui_state.set_max_texture_side(max_texture_side);
        }

        window.request_redraw();
        self.egui_state = Some(egui_state);
        self.painter = Some(painter);
        self.window = Some(window);
        Ok(())
    }

    fn render(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(egui_state) = self.egui_state.as_mut() else {
            return;
        };
        let Some(painter) = self.painter.as_mut() else {
            return;
        };

        let raw_input = egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |ctx| {
            show_node_editor(ctx, &mut self.graph_editor);
        });

        egui_state.handle_platform_output(window, output.platform_output);

        let clipped_primitives = self
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point);

        painter.paint_and_update_textures(
            ViewportId::ROOT,
            output.pixels_per_point,
            [0.07, 0.08, 0.10, 1.0],
            &clipped_primitives,
            &output.textures_delta,
            Vec::new(),
        );

        if self.egui_ctx.has_requested_repaint() {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for NodeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(error) = self.create_window(event_loop) {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        if let Some(egui_state) = self.egui_state.as_mut() {
            let response = egui_state.on_window_event(window, &event);
            if response.repaint {
                window.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height), Some(painter)) = (
                    NonZeroU32::new(size.width),
                    NonZeroU32::new(size.height),
                    self.painter.as_mut(),
                ) {
                    painter.on_window_resized(ViewportId::ROOT, width, height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
}
