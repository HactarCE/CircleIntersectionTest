use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, PI, TAU};

use eframe::egui::{Painter, Pos2, Stroke, pos2, vec2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Circle {
    pub center: Pos2,
    pub radius: f32,
}
impl Circle {
    pub fn draw(self, p: &Painter, stroke: impl Into<Stroke>) {
        p.circle_stroke(self.center, self.radius, stroke);
    }

    pub fn to_cga(self) -> cga2d::Blade3 {
        cga2d::circle(
            cga2d::point(self.center.x as f64, self.center.y as f64),
            self.radius as f64,
        )
    }

    pub fn from_cga(blade: cga2d::Blade3) -> Option<Self> {
        match blade.unpack(crate::EPSILON) {
            cga2d::LineOrCircle::Line { .. } => None,
            cga2d::LineOrCircle::Circle { cx, cy, r } => Some(Self {
                center: pos2(cx as f32, cy as f32),
                radius: r as f32,
            }),
        }
    }

    pub fn point_at_angle(self, angle: f32) -> Pos2 {
        let (sin, cos) = angle.sin_cos();
        self.center + vec2(cos, sin) * self.radius
    }

    pub fn contains(self, pos: Pos2) -> bool {
        self.center.distance_sq(pos) < self.radius * self.radius
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ArcSegment {
    pub circle: Circle,
    pub start_angle: f32,
    pub end_angle: f32,
}
impl ArcSegment {
    pub fn start_point(self) -> Pos2 {
        self.circle.point_at_angle(self.start_angle)
    }
    pub fn end_point(self) -> Pos2 {
        self.circle.point_at_angle(self.end_angle)
    }

    fn radians(self) -> f32 {
        (self.end_angle - self.start_angle).rem_euclid(TAU)
    }

    pub fn midpoint(self) -> Pos2 {
        self.circle
            .point_at_angle(self.start_angle + self.radians() / 2.0)
    }

    pub fn points_for_drawing(self) -> impl Iterator<Item = Pos2> {
        let start = self.start_angle;
        let end = self.start_angle + self.radians();
        let point_count = (crate::CIRCLE_POLYGON_POINTS as f32 * (end - start) / TAU).ceil();
        // exclude the very last point (because it'll be covered by the next arc)
        (0..point_count as usize)
            .map(move |i| lerp(start, end, i as f32 / point_count))
            .map(move |angle| self.circle.point_at_angle(angle))
    }
}

pub fn cut_circle_by_circles(
    circle: Circle,
    others: impl IntoIterator<Item = Circle>,
) -> Vec<ArcSegment> {
    let c = circle.to_cga();
    let mut split_angles = vec![];
    for other in others {
        let o = other.to_cga();
        if let Some(points) = (c & o).unpack_point_pair() {
            for p in points {
                let (x, y) = p.unpack_point();
                split_angles.push(
                    (pos2(x as f32, y as f32) - circle.center)
                        .angle()
                        .rem_euclid(TAU),
                );
            }
        }
    }
    split_angles.sort_by(f32::total_cmp);
    let Some(first) = split_angles.first() else {
        return vec![ArcSegment {
            circle,
            start_angle: 0.0,
            end_angle: TAU,
        }];
    };

    let offset_split_angles = split_angles.iter().skip(1).chain([first]);

    std::iter::zip(&split_angles, offset_split_angles)
        .map(|(&start_angle, &end_angle)| ArcSegment {
            circle,
            start_angle,
            end_angle,
        })
        .collect()
}

fn iter_excluding_index<T>(
    iter: impl IntoIterator<Item = T>,
    index_to_exclude: usize,
) -> impl Iterator<Item = T> {
    iter.into_iter()
        .enumerate()
        .filter(move |(i, _)| *i != index_to_exclude)
        .map(|(_, t)| t)
}

pub fn intersect_many_circles(circles: &[Circle]) -> Vec<ArcSegment> {
    let mut all_segments = vec![];
    for (i, &circle) in circles.iter().enumerate() {
        let candidate_segments =
            cut_circle_by_circles(circle, iter_excluding_index(circles, i).copied());
        let new_segments = candidate_segments.into_iter().filter(|segment| {
            let arc_interior_point = segment.midpoint();
            iter_excluding_index(circles, i).all(|other| other.contains(arc_interior_point))
        });
        all_segments.extend(new_segments);
    }
    all_segments
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}
