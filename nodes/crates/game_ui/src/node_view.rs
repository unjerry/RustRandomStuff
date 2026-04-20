use egui::{
    pos2, vec2, Align2, Button, CentralPanel, Color32, CornerRadius, FontId, Id, Rect, Sense,
    Stroke, StrokeKind, TopBottomPanel, Ui,
};
use nodes_core::{Color, NodeTemplate, PortDirection, PortKind, PortTemplate, PropertyTemplate};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeInstanceId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeRenderPlan {
    pub template_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub size: Size,
    pub header_height: f32,
    pub header_color: Color,
    pub body_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub ports: Vec<PortRenderPlan>,
    pub properties: Vec<PropertyRenderPlan>,
}

impl NodeRenderPlan {
    pub fn from_template(template: &NodeTemplate) -> Self {
        let input_count = template
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Input)
            .count();
        let output_count = template
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Output)
            .count();
        let port_rows = input_count.max(output_count).max(1);
        let property_rows = template.properties.len();

        let size = template.size;
        let port_area_height = port_rows as f32 * size.row_height;
        let property_gap = if property_rows == 0 {
            0.0
        } else {
            size.padding
        };
        let property_area_height = property_rows as f32 * size.row_height;
        let height = size.header_height
            + size.padding
            + port_area_height
            + property_gap
            + property_area_height
            + size.padding;

        let mut input_row = 0;
        let mut output_row = 0;
        let ports = template
            .ports
            .iter()
            .map(|port| {
                let row = match port.direction {
                    PortDirection::Input => {
                        let row = input_row;
                        input_row += 1;
                        row
                    }
                    PortDirection::Output => {
                        let row = output_row;
                        output_row += 1;
                        row
                    }
                };

                port_render_plan(template, port, row)
            })
            .collect();

        let property_y = size.header_height + size.padding + port_area_height + property_gap;
        let properties = template
            .properties
            .iter()
            .enumerate()
            .map(|(index, property)| property_render_plan(template, property, property_y, index))
            .collect();

        Self {
            template_id: template.id.clone(),
            title: template.title.clone(),
            subtitle: template.subtitle.clone(),
            size: Size::new(size.width, height),
            header_height: size.header_height,
            header_color: template.style.header_color,
            body_color: template.style.body_color,
            border_color: template.style.border_color,
            text_color: template.style.text_color,
            ports,
            properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortRenderPlan {
    pub id: String,
    pub label: String,
    pub direction: PortDirection,
    pub kind: PortKind,
    pub item: Option<String>,
    pub capacity_per_tick: Option<u64>,
    pub socket_center: Point,
    pub socket_radius: f32,
    pub socket_color: Color,
    pub label_anchor: Point,
    pub label_align: TextAlign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyRenderPlan {
    pub id: String,
    pub label: String,
    pub value: String,
    pub label_anchor: Point,
    pub value_anchor: Point,
}

#[derive(Debug, Clone)]
pub struct NodeGraphEditor {
    prototype: NodeRenderPlan,
    nodes: Vec<NodeInstance>,
    links: Vec<NodeLink>,
    pending_link: Option<PendingLink>,
    next_node_id: u64,
}

impl NodeGraphEditor {
    pub fn new(prototype: NodeRenderPlan) -> Self {
        let mut editor = Self {
            prototype,
            nodes: Vec::new(),
            links: Vec::new(),
            pending_link: None,
            next_node_id: 0,
        };

        editor.add_node_at(Point::new(160.0, 140.0));
        editor.add_node_at(Point::new(560.0, 240.0));
        editor
    }

    pub fn add_node(&mut self) -> NodeInstanceId {
        let index = self.nodes.len();
        let column = index % 3;
        let row = index / 3;
        self.add_node_at(Point::new(
            140.0 + column as f32 * 320.0,
            130.0 + row as f32 * 230.0,
        ))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    fn show(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("node_toolbar")
            .exact_height(44.0)
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(24, 28, 33)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui
                        .add(Button::new("+ Node").min_size(vec2(96.0, 28.0)))
                        .clicked()
                    {
                        self.add_node();
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(12.0);
                    ui.label(format!(
                        "{} nodes  {} links",
                        self.node_count(),
                        self.link_count()
                    ));
                });
            });

        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(18, 21, 25)))
            .show(ctx, |ui| {
                draw_grid(ui);
                self.draw_links(ui);

                let mut hovered_input = None;
                let mut started_output = None;

                for node in &mut self.nodes {
                    let interaction = draw_node_instance(ui, node);
                    hovered_input = hovered_input.or(interaction.hovered_input);
                    started_output = started_output.or(interaction.started_output);
                }

                if let Some(endpoint) = started_output {
                    self.pending_link = Some(PendingLink { from: endpoint });
                    ui.ctx().request_repaint();
                }

                self.draw_pending_link(ui);

                let pointer_released = ui.ctx().input(|input| input.pointer.any_released());
                if pointer_released {
                    if let Some(pending) = self.pending_link.take() {
                        if let Some(target) = hovered_input {
                            self.try_connect(pending.from, target);
                        }
                    }
                }
            });
    }

    fn add_node_at(&mut self, position: Point) -> NodeInstanceId {
        let id = NodeInstanceId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.push(NodeInstance {
            id,
            plan: self.prototype.clone(),
            position,
        });
        id
    }

    fn draw_links(&mut self, ui: &Ui) {
        let mut action = None;

        for link_index in 0..self.links.len() {
            let Some(points) = self.link_points(link_index) else {
                continue;
            };

            draw_link_path(ui, &points, Color32::from_rgb(122, 202, 146), 3.0);

            if action.is_none() {
                action = hit_test_link_segments(ui, link_index, &points);
            }

            if let Some(link) = self.links.get_mut(link_index) {
                for waypoint_index in 0..link.waypoints.len() {
                    let point = link.waypoints[waypoint_index];
                    let center = pos2(point.x, point.y);
                    let hit_rect = Rect::from_center_size(center, vec2(22.0, 22.0));
                    let response = ui.interact(
                        hit_rect,
                        Id::new(("link_waypoint", link_index, waypoint_index)),
                        Sense::click_and_drag(),
                    );

                    if response.dragged() {
                        let delta = response.drag_delta();
                        link.waypoints[waypoint_index].x += delta.x;
                        link.waypoints[waypoint_index].y += delta.y;
                        ui.ctx().request_repaint();
                    }

                    if response.secondary_clicked() {
                        action = Some(LinkAction::RemoveWaypoint {
                            link_index,
                            waypoint_index,
                        });
                    }

                    let color = if response.hovered() || response.dragged() {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(122, 202, 146)
                    };
                    ui.painter().circle_filled(center, 5.0, color);
                    ui.painter().circle_stroke(
                        center,
                        8.0,
                        Stroke::new(1.0, Color32::from_rgb(22, 26, 30)),
                    );
                }
            }
        }

        if let Some(action) = action {
            self.apply_link_action(action);
            ui.ctx().request_repaint();
        }
    }

    fn draw_pending_link(&self, ui: &Ui) {
        let Some(pending) = &self.pending_link else {
            return;
        };
        let Some(from) = self.endpoint_center(&pending.from) else {
            return;
        };
        let Some(pointer) = ui.ctx().pointer_hover_pos() else {
            return;
        };

        draw_link_path(
            ui,
            &[from, pointer],
            Color32::from_rgba_unmultiplied(180, 220, 190, 190),
            2.0,
        );
        ui.ctx().request_repaint();
    }

    fn apply_link_action(&mut self, action: LinkAction) {
        match action {
            LinkAction::Cut { link_index } => {
                if link_index < self.links.len() {
                    self.links.remove(link_index);
                }
            }
            LinkAction::AddWaypoint {
                link_index,
                segment_index,
                position,
            } => {
                if let Some(link) = self.links.get_mut(link_index) {
                    let insertion_index = segment_index.min(link.waypoints.len());
                    link.waypoints
                        .insert(insertion_index, Point::new(position.x, position.y));
                }
            }
            LinkAction::RemoveWaypoint {
                link_index,
                waypoint_index,
            } => {
                if let Some(link) = self.links.get_mut(link_index) {
                    if waypoint_index < link.waypoints.len() {
                        link.waypoints.remove(waypoint_index);
                    }
                }
            }
        }
    }

    fn try_connect(&mut self, from: Endpoint, to: Endpoint) {
        if from.node_id == to.node_id {
            return;
        }

        self.links.retain(|link| link.to != to);

        if self
            .links
            .iter()
            .any(|link| link.from == from && link.to == to)
        {
            return;
        }

        self.links.push(NodeLink {
            from,
            to,
            waypoints: Vec::new(),
        });
    }

    fn link_points(&self, link_index: usize) -> Option<Vec<egui::Pos2>> {
        let link = self.links.get(link_index)?;
        let mut points = Vec::with_capacity(link.waypoints.len() + 2);
        points.push(self.endpoint_center(&link.from)?);
        points.extend(link.waypoints.iter().map(|point| pos2(point.x, point.y)));
        points.push(self.endpoint_center(&link.to)?);
        Some(points)
    }

    fn endpoint_center(&self, endpoint: &Endpoint) -> Option<egui::Pos2> {
        let node = self.nodes.iter().find(|node| node.id == endpoint.node_id)?;
        let port = node
            .plan
            .ports
            .iter()
            .find(|port| port.id == endpoint.port_id)?;

        Some(pos2(
            node.position.x + port.socket_center.x,
            node.position.y + port.socket_center.y,
        ))
    }
}

pub fn show_node_editor(ctx: &egui::Context, editor: &mut NodeGraphEditor) {
    editor.show(ctx);
}

#[derive(Debug, Clone)]
struct NodeInstance {
    id: NodeInstanceId,
    plan: NodeRenderPlan,
    position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    node_id: NodeInstanceId,
    port_id: String,
}

#[derive(Debug, Clone)]
struct NodeLink {
    from: Endpoint,
    to: Endpoint,
    waypoints: Vec<Point>,
}

#[derive(Debug, Clone)]
struct PendingLink {
    from: Endpoint,
}

#[derive(Debug, Clone, Copy)]
enum LinkAction {
    Cut {
        link_index: usize,
    },
    AddWaypoint {
        link_index: usize,
        segment_index: usize,
        position: egui::Pos2,
    },
    RemoveWaypoint {
        link_index: usize,
        waypoint_index: usize,
    },
}

#[derive(Debug, Default)]
struct NodeInteraction {
    hovered_input: Option<Endpoint>,
    started_output: Option<Endpoint>,
}

fn draw_node_instance(ui: &mut Ui, node: &mut NodeInstance) -> NodeInteraction {
    let size = vec2(node.plan.size.width, node.plan.size.height);
    let rect = Rect::from_min_size(pos2(node.position.x, node.position.y), size);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), node.plan.header_height));
    let drag_response = ui.interact(
        header_rect,
        Id::new(("node_drag", node.id.0)),
        Sense::click_and_drag(),
    );

    if drag_response.dragged() {
        let delta = drag_response.drag_delta();
        node.position.x += delta.x;
        node.position.y += delta.y;
        ui.ctx().request_repaint();
    }

    let rect = Rect::from_min_size(pos2(node.position.x, node.position.y), size);
    draw_node(
        ui,
        rect,
        &node.plan,
        drag_response.hovered() || drag_response.dragged(),
    );

    let mut interaction = NodeInteraction::default();
    for port in &node.plan.ports {
        let center = rect.min + vec2(port.socket_center.x, port.socket_center.y);
        let hit_radius = port.socket_radius + 10.0;
        let hit_rect = Rect::from_center_size(center, vec2(hit_radius * 2.0, hit_radius * 2.0));
        let response = ui.interact(
            hit_rect,
            Id::new(("port", node.id.0, port.id.as_str())),
            Sense::click_and_drag(),
        );

        if response.hovered() || response.dragged() {
            ui.painter().circle_stroke(
                center,
                port.socket_radius + 5.0,
                Stroke::new(2.0, Color32::WHITE),
            );
        }

        let endpoint = Endpoint {
            node_id: node.id,
            port_id: port.id.clone(),
        };

        if port.direction == PortDirection::Output && response.drag_started() {
            interaction.started_output = Some(endpoint.clone());
        }

        if port.direction == PortDirection::Input && response.hovered() {
            interaction.hovered_input = Some(endpoint);
        }
    }

    interaction
}

fn draw_node(ui: &Ui, rect: Rect, plan: &NodeRenderPlan, active: bool) {
    let painter = ui.painter();
    let radius = CornerRadius::same(6);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), plan.header_height));

    painter.rect_filled(rect, radius, color32(plan.body_color));
    painter.rect_filled(
        header_rect,
        CornerRadius {
            nw: 6,
            ne: 6,
            sw: 0,
            se: 0,
        },
        color32(plan.header_color),
    );
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(if active { 2.0 } else { 1.0 }, color32(plan.border_color)),
        StrokeKind::Inside,
    );

    let text_color = color32(plan.text_color);
    painter.text(
        rect.min + vec2(12.0, plan.header_height * 0.5),
        Align2::LEFT_CENTER,
        &plan.title,
        FontId::proportional(16.0),
        text_color,
    );

    if let Some(subtitle) = &plan.subtitle {
        painter.text(
            rect.right_top() + vec2(-12.0, plan.header_height * 0.5),
            Align2::RIGHT_CENTER,
            subtitle,
            FontId::proportional(12.0),
            Color32::from_rgba_unmultiplied(text_color.r(), text_color.g(), text_color.b(), 180),
        );
    }

    for port in &plan.ports {
        let center = rect.min + vec2(port.socket_center.x, port.socket_center.y);
        painter.circle_filled(center, port.socket_radius + 2.0, color32(plan.border_color));
        painter.circle_filled(center, port.socket_radius, color32(port.socket_color));

        let label_anchor = rect.min + vec2(port.label_anchor.x, port.label_anchor.y);
        let align = match port.label_align {
            TextAlign::Left => Align2::LEFT_CENTER,
            TextAlign::Right => Align2::RIGHT_CENTER,
        };
        painter.text(
            label_anchor,
            align,
            &port.label,
            FontId::proportional(13.0),
            text_color,
        );
    }

    if !plan.properties.is_empty() {
        let first_property_y = plan.properties[0].label_anchor.y - 14.0;
        painter.line_segment(
            [
                rect.min + vec2(12.0, first_property_y),
                rect.min + vec2(rect.width() - 12.0, first_property_y),
            ],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
        );
    }

    for property in &plan.properties {
        painter.text(
            rect.min + vec2(property.label_anchor.x, property.label_anchor.y),
            Align2::LEFT_CENTER,
            &property.label,
            FontId::proportional(12.0),
            Color32::from_rgba_unmultiplied(text_color.r(), text_color.g(), text_color.b(), 170),
        );
        painter.text(
            rect.min + vec2(property.value_anchor.x, property.value_anchor.y),
            Align2::RIGHT_CENTER,
            &property.value,
            FontId::proportional(12.0),
            text_color,
        );
    }
}

fn draw_link_path(ui: &Ui, points: &[egui::Pos2], color: Color32, width: f32) {
    for segment in points.windows(2) {
        ui.painter()
            .line_segment([segment[0], segment[1]], Stroke::new(width, color));
    }
}

fn hit_test_link_segments(ui: &Ui, link_index: usize, points: &[egui::Pos2]) -> Option<LinkAction> {
    let pointer = ui.ctx().pointer_hover_pos()?;

    for (segment_index, segment) in points.windows(2).enumerate() {
        let start = segment[0];
        let end = segment[1];
        let hit_rect = Rect::from_two_pos(start, end).expand(8.0);
        let response = ui.interact(
            hit_rect,
            Id::new(("link_segment", link_index, segment_index)),
            Sense::click(),
        );
        let is_near = distance_to_segment(pointer, start, end) <= 8.0;

        if response.hovered() && is_near {
            ui.painter().line_segment(
                [start, end],
                Stroke::new(5.0, Color32::from_rgba_unmultiplied(255, 255, 255, 50)),
            );
        }

        if response.secondary_clicked() && is_near {
            return Some(LinkAction::Cut { link_index });
        }

        if response.double_clicked() && is_near {
            return Some(LinkAction::AddWaypoint {
                link_index,
                segment_index,
                position: pointer,
            });
        }
    }

    None
}

fn distance_to_segment(point: egui::Pos2, start: egui::Pos2, end: egui::Pos2) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;

    if length_squared <= f32::EPSILON {
        return point.distance(start);
    }

    let t =
        (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared).clamp(0.0, 1.0);
    let closest = pos2(start.x + t * dx, start.y + t * dy);
    point.distance(closest)
}

fn draw_grid(ui: &mut Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let minor = 24.0;
    let major = minor * 4.0;

    draw_grid_lines(
        painter,
        rect,
        minor,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14)),
    );
    draw_grid_lines(
        painter,
        rect,
        major,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
    );
}

fn draw_grid_lines(painter: &egui::Painter, rect: Rect, spacing: f32, stroke: Stroke) {
    let start_x = (rect.left() / spacing).floor() * spacing;
    let start_y = (rect.top() / spacing).floor() * spacing;

    let mut x = start_x;
    while x <= rect.right() {
        painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
        x += spacing;
    }

    let mut y = start_y;
    while y <= rect.bottom() {
        painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
        y += spacing;
    }
}

fn port_render_plan(template: &NodeTemplate, port: &PortTemplate, row: usize) -> PortRenderPlan {
    let size = template.size;
    let center_y =
        size.header_height + size.padding + row as f32 * size.row_height + size.row_height * 0.5;

    let (socket_x, label_x, label_align) = match port.direction {
        PortDirection::Input => (0.0, size.padding + size.port_radius + 8.0, TextAlign::Left),
        PortDirection::Output => (
            size.width,
            size.width - size.padding - size.port_radius - 8.0,
            TextAlign::Right,
        ),
    };

    PortRenderPlan {
        id: port.id.clone(),
        label: port.label.clone(),
        direction: port.direction,
        kind: port.kind,
        item: port.item.clone(),
        capacity_per_tick: port.capacity_per_tick,
        socket_center: Point::new(socket_x, center_y),
        socket_radius: size.port_radius,
        socket_color: port_color(template, port.kind),
        label_anchor: Point::new(label_x, center_y),
        label_align,
    }
}

fn property_render_plan(
    template: &NodeTemplate,
    property: &PropertyTemplate,
    start_y: f32,
    row: usize,
) -> PropertyRenderPlan {
    let size = template.size;
    let center_y = start_y + row as f32 * size.row_height + size.row_height * 0.5;

    PropertyRenderPlan {
        id: property.id.clone(),
        label: property.label.clone(),
        value: property_value_text(property),
        label_anchor: Point::new(size.padding, center_y),
        value_anchor: Point::new(size.width - size.padding, center_y),
    }
}

fn port_color(template: &NodeTemplate, kind: PortKind) -> Color {
    match kind {
        PortKind::Item => template.style.item_port_color,
        PortKind::Energy => template.style.energy_port_color,
        PortKind::Signal => template.style.signal_port_color,
    }
}

fn property_value_text(property: &PropertyTemplate) -> String {
    let mut value = match &property.value {
        nodes_core::TemplateValue::Bool(value) => value.to_string(),
        nodes_core::TemplateValue::Number(value) => value.to_string(),
        nodes_core::TemplateValue::Text(value) => value.clone(),
    };

    if let Some(unit) = &property.unit {
        value.push(' ');
        value.push_str(unit);
    }

    value
}

fn color32(color: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nodes_core::NodeTemplate;

    #[test]
    fn lays_out_sample_ports_on_node_edges() {
        let template = NodeTemplate::from_json_str(include_str!(
            "../../../assets/node_templates/ore_source.json"
        ))
        .unwrap();

        let plan = NodeRenderPlan::from_template(&template);

        assert_eq!(plan.size.width, 260.0);
        assert!(plan.size.height > 0.0);
        assert_eq!(plan.ports[0].socket_center.x, 0.0);
        assert_eq!(plan.ports[2].socket_center.x, 260.0);
    }

    #[test]
    fn graph_editor_starts_with_two_nodes() {
        let template = NodeTemplate::from_json_str(include_str!(
            "../../../assets/node_templates/ore_source.json"
        ))
        .unwrap();
        let plan = NodeRenderPlan::from_template(&template);
        let editor = NodeGraphEditor::new(plan);

        assert_eq!(editor.node_count(), 2);
        assert_eq!(editor.link_count(), 0);
    }
}
