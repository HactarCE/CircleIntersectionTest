use cga2d::*;
use eframe::egui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Circle Intersection Test",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Circle {
    center: Pos2,
    radius: f32,
}
impl Circle {
    pub fn draw(self, p: &Painter, stroke: impl Into<Stroke>) {
        p.circle_stroke(self.center, self.radius, stroke);
    }
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
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
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
        });
    }
}

fn color(i: usize, n: usize) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i, n);
    Color32::from_rgb(r, g, b)
}
