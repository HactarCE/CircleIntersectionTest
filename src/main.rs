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
    drag_start: Option<Pos2>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);
        Self {
            circles: vec![],
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
            for i in 0..self.circles.len() {
                ui.horizontal(|ui| {
                    if ui.button("🗑").clicked() {
                        to_delete = Some(i);
                    }
                    let h = ui.min_rect().height();
                    let (rect, _) = ui.allocate_exact_size(vec2(h, h), Sense::empty());
                    let stroke = (1.0, Color32::WHITE);
                    ui.painter()
                        .rect(rect, 3.0, color(i, n), stroke, StrokeKind::Inside);
                    ui.label(format!("Circle {}", i + 1));
                });
            }
            if let Some(i) = to_delete {
                self.circles.remove(i);
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            let r = ui.allocate_rect(ui.available_rect_before_wrap(), Sense::click_and_drag());
            let p = ui.painter();
            if r.drag_started() {
                self.drag_start = r.interact_pointer_pos();
            }

            let mut circs = self.circles.clone();

            if let Some((drag_start, drag_end)) =
                Option::zip(self.drag_start, r.interact_pointer_pos())
            {
                let new_circ = Circle {
                    center: drag_start,
                    radius: drag_start.distance(drag_end),
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
                c.draw(p, (1.0, color(i, n)));
            }

            let mut arcs = circle::intersect_many_circles(&circs);
            let Some(first_arc) = arcs.pop() else { return };

            let mut arcs_in_order = vec![first_arc];
            let mut last_point = first_arc.end_point();

            while let Some(i) = arcs.iter().position(|a| {
                f32::min(
                    a.start_point().distance_sq(last_point),
                    a.end_point().distance_sq(last_point),
                ) < EPSILON as f32
            }) {
                let new_arc = arcs.swap_remove(i);
                arcs_in_order.push(new_arc);
                last_point = new_arc.end_point();
            }

            let polygon_points = arcs_in_order
                .into_iter()
                .flat_map(|arc| arc.points_for_drawing())
                .collect();
            p.add(Shape::convex_polygon(
                polygon_points,
                Color32::from_rgba_unmultiplied(200, 0, 100, 100),
                (1.5, Color32::WHITE),
            ));
            // if circs.is_empty() {
            //     return;
            // }
            // let arcs = circle::cut_circle_by_circles(circs[0], circs[1..].iter().copied());
            // let n = arcs.len();
            // for (i, &arc) in arcs.iter().enumerate() {
            //     p.add(Shape::convex_polygon(
            //         arc.points_for_drawing().collect(),
            //         color(i, n),
            //         (1.5, Color32::WHITE),
            //     ));
            //     p.circle(arc.midpoint(), 2.0, color(i, n), (0.5, Color32::BLACK));
            // }
        });
    }
}

fn color(i: usize, n: usize) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i, n);
    Color32::from_rgb(r, g, b)
}
