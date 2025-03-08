use eframe::egui::*;

mod circle;

use circle::Circle;

/// Precision for floating-point comparisons.
pub const EPSILON: f64 = 0.001;
/// Number of points to use when rendering a 360-degrees circular arc as a
/// polygon.
pub const CIRCLE_POLYGON_POINTS: f64 = 50.0;

fn main() -> eframe::Result {
    eframe::run_native(
        "Circle Intersection Test",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    circles: Vec<Circle>,
    circles_enabled: Vec<bool>,
    drag_start: Option<Pos2>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.5);
        Self {
            circles: vec![],
            circles_enabled: vec![],
            drag_start: None,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::right("right_panel").show(ctx, |ui| {
            // Show circles list
            let n = self.circles.len();
            let mut to_delete = None;
            self.circles_enabled.resize(n, true);
            for i in 0..n {
                ui.horizontal(|ui| {
                    if ui.button("🗑").clicked() {
                        to_delete = Some(i);
                    }
                    ui.checkbox(&mut self.circles_enabled[i], "");
                    let h = ui.min_rect().height();
                    let (rect, _) = ui.allocate_exact_size(vec2(h, h), Sense::empty());
                    let stroke = (1.0, Color32::WHITE);
                    ui.painter()
                        .rect(rect, 3.0, color(i, n), stroke, StrokeKind::Inside);
                    ui.label(format!("Circle {}", i + 1));
                });
                ui.checkbox(&mut self.circles[i].inverted, "Inverted");
                ui.separator();
            }
            ui.label("Click and drag to add a new circle");
            if let Some(i) = to_delete {
                self.circles.remove(i);
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            let r = ui.interact(
                ui.available_rect_before_wrap(),
                Id::new("circle_interaction"),
                Sense::click_and_drag(),
            );
            if r.drag_started() {
                self.drag_start = r.interact_pointer_pos();
            }

            let mut circs = self
                .circles
                .iter()
                .zip(&self.circles_enabled)
                .filter(|(_, enabled)| **enabled)
                .map(|(c, _)| *c)
                .collect::<Vec<_>>();

            if let Some((drag_start, drag_end)) =
                Option::zip(self.drag_start, r.interact_pointer_pos())
            {
                let new_circ = Circle {
                    center: drag_start,
                    radius: drag_start.distance(drag_end),
                    inverted: false,
                };
                if r.dragged() || r.drag_stopped() {
                    circs.push(new_circ);
                }
                if r.drag_stopped() {
                    self.circles.push(new_circ);
                }
            }

            let n = circs.len();

            for (i, c) in circs.iter().enumerate() {
                c.draw(ui.painter(), (1.0, color(i, n)));
            }

            let mut arcs = circle::intersect_many_circles(&circs);

            while !arcs.is_empty() {
                let Some(first_arc) = arcs.pop() else { return };

                let mut arcs_in_order = vec![first_arc];
                let mut last_point = if first_arc.circle.inverted {
                    first_arc.start_point()
                } else {
                    first_arc.end_point()
                };

                while let Some(i) = arcs.iter().position(|a| {
                    if a.circle.inverted {
                        a.end_point()
                    } else {
                        a.start_point()
                    }
                    .distance_sq(last_point)
                        < EPSILON as f32
                }) {
                    let new_arc = arcs.swap_remove(i);
                    arcs_in_order.push(new_arc);
                    last_point = if new_arc.circle.inverted {
                        new_arc.start_point()
                    } else {
                        new_arc.end_point()
                    };
                }

                if arcs_in_order.is_empty() {
                    continue;
                }

                let polygon_points = arcs_in_order
                    .into_iter()
                    .flat_map(|arc| arc.points_for_drawing())
                    .collect();
                ui.painter().add(Shape::convex_polygon(
                    polygon_points,
                    Color32::from_rgba_unmultiplied(200, 0, 100, 10),
                    (1.5, Color32::WHITE),
                ));
            }
        });
    }
}

fn color(i: usize, n: usize) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i, n);
    Color32::from_rgb(r, g, b)
}
