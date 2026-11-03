use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

fn s(rect: Rect, x: f32, y: f32) -> Pos2 {
    Pos2::new(
        rect.left() + rect.width() * x,
        rect.top() + rect.height() * y,
    )
}

fn line(painter: &Painter, rect: Rect, points: &[(f32, f32)], color: Color32, width: f32) {
    let points: Vec<Pos2> = points.iter().map(|(x, y)| s(rect, *x, *y)).collect();
    painter.add(Shape::line(points, Stroke::new(width, color)));
}

pub fn power(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.33;
    let points: Vec<Pos2> = (0..=40)
        .map(|index| {
            let angle = -1.15 + (index as f32 / 40.0) * (std::f32::consts::TAU - 1.05);
            center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(width, color)));
    painter.line_segment(
        [
            Pos2::new(center.x, rect.top() + rect.height() * 0.16),
            Pos2::new(center.x, center.y - radius * 0.1),
        ],
        Stroke::new(width, color),
    );
}

pub fn route(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let left = s(rect, 0.18, 0.60);
    let right = s(rect, 0.82, 0.40);
    painter.circle_filled(left, width * 1.7, color);
    painter.circle_filled(right, width * 1.7, color);
    let mut points = Vec::new();
    for index in 0..=28 {
        let t = index as f32 / 28.0;
        let one = 1.0 - t;
        let x = 0.18 * one * one * one
            + 3.0 * 0.30 * one * one * t
            + 3.0 * 0.70 * one * t * t
            + 0.82 * t * t * t;
        let y = 0.62 * one * one * one
            + 3.0 * 0.16 * one * one * t
            + 3.0 * 0.84 * one * t * t
            + 0.38 * t * t * t;
        points.push(s(rect, x, y));
    }
    painter.add(Shape::line(points, Stroke::new(width, color)));
}

pub fn search(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let center = s(rect, 0.43, 0.43);
    let radius = rect.width() * 0.24;
    painter.circle_stroke(center, radius, Stroke::new(width, color));
    painter.line_segment(
        [
            center + Vec2::angled(std::f32::consts::FRAC_PI_4) * radius,
            s(rect, 0.80, 0.80),
        ],
        Stroke::new(width, color),
    );
}

pub fn log(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    for (index, y) in [0.30, 0.50, 0.70].iter().enumerate() {
        painter.circle_filled(s(rect, 0.20, *y), width * 0.9, color);
        let end = if index == 1 { 0.78 } else { 0.82 };
        line(painter, rect, &[(0.36, *y), (end, *y)], color, width * 0.9);
    }
}

pub fn cog(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let center = rect.center();
    let scale = rect.width().min(rect.height());
    let outer = scale * 0.37;
    let root = scale * 0.24;
    let mut points = Vec::with_capacity(24);
    for index in 0..6 {
        let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::TAU / 6.0;
        for (offset, radius) in [(-0.28, root), (-0.17, outer), (0.17, outer), (0.28, root)] {
            points.push(center + Vec2::angled(angle + offset) * radius);
        }
    }
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    painter.circle_stroke(center, scale * 0.105, Stroke::new(width, color));
}

pub fn pin(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let center = s(rect, 0.50, 0.40);
    let radius = rect.width() * 0.20;
    painter.circle_stroke(center, radius, Stroke::new(width, color));
    painter.circle_filled(center, width * 1.2, color);
    line(
        painter,
        rect,
        &[(0.36, 0.54), (0.50, 0.84), (0.64, 0.54)],
        color,
        width,
    );
}

pub fn globe(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let center = s(rect, 0.50, 0.50);
    let radius = rect.width() * 0.32;
    painter.circle_stroke(center, radius, Stroke::new(width, color));
    let rx = radius * 0.45;
    painter.add(Shape::ellipse_stroke(
        center,
        Vec2::new(rx, radius),
        Stroke::new(width * 0.85, color),
    ));
    line(
        painter,
        rect,
        &[(0.18, 0.50), (0.82, 0.50)],
        color,
        width * 0.85,
    );
    line(
        painter,
        rect,
        &[(0.24, 0.35), (0.76, 0.35)],
        color,
        width * 0.75,
    );
    line(
        painter,
        rect,
        &[(0.24, 0.65), (0.76, 0.65)],
        color,
        width * 0.75,
    );
}

pub fn import(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    line(painter, rect, &[(0.50, 0.18), (0.50, 0.62)], color, width);
    line(
        painter,
        rect,
        &[(0.34, 0.48), (0.50, 0.64), (0.66, 0.48)],
        color,
        width,
    );
    line(
        painter,
        rect,
        &[(0.22, 0.72), (0.22, 0.82), (0.78, 0.82), (0.78, 0.72)],
        color,
        width,
    );
}

pub fn copy(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let back = Rect::from_min_max(s(rect, 0.44, 0.12), s(rect, 0.92, 0.60));
    let front = Rect::from_min_max(s(rect, 0.12, 0.40), s(rect, 0.60, 0.88));
    painter.rect_stroke(
        back,
        egui::CornerRadius::same(1),
        Stroke::new(width, color),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        front,
        egui::CornerRadius::same(1),
        Stroke::new(width, color),
        egui::StrokeKind::Inside,
    );
}

pub fn folder(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let points = [
        (0.16, 0.34),
        (0.40, 0.34),
        (0.48, 0.26),
        (0.84, 0.26),
        (0.84, 0.74),
        (0.16, 0.74),
    ];
    let points: Vec<Pos2> = points.iter().map(|(x, y)| s(rect, *x, *y)).collect();
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
}

pub fn trash(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    line(painter, rect, &[(0.20, 0.28), (0.80, 0.28)], color, width);
    line(painter, rect, &[(0.40, 0.20), (0.60, 0.20)], color, width);
    line(
        painter,
        rect,
        &[(0.28, 0.28), (0.32, 0.82), (0.68, 0.82), (0.72, 0.28)],
        color,
        width,
    );
    line(
        painter,
        rect,
        &[(0.44, 0.38), (0.45, 0.70)],
        color,
        width * 0.8,
    );
    line(
        painter,
        rect,
        &[(0.56, 0.38), (0.55, 0.70)],
        color,
        width * 0.8,
    );
}

pub fn close(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    line(painter, rect, &[(0.28, 0.28), (0.72, 0.72)], color, width);
    line(painter, rect, &[(0.72, 0.28), (0.28, 0.72)], color, width);
}

pub fn eye(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let mut points = Vec::new();
    for index in 0..=24 {
        let t = index as f32 / 24.0;
        let x = 0.16 + t * 0.68;
        let y = 0.50 - (t * std::f32::consts::PI).sin() * 0.22;
        points.push(s(rect, x, y));
    }
    for index in (0..=24).rev() {
        let t = index as f32 / 24.0;
        let x = 0.16 + t * 0.68;
        let y = 0.50 + (t * std::f32::consts::PI).sin() * 0.22;
        points.push(s(rect, x, y));
    }
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    painter.circle_filled(s(rect, 0.50, 0.50), width * 2.2, color);
}

pub fn eye_off(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    eye(painter, rect, color, width);
    line(painter, rect, &[(0.20, 0.20), (0.80, 0.80)], color, width);
}

pub fn star(painter: &Painter, rect: Rect, color: Color32, width: f32, filled: bool) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.36;
    let mut points = Vec::new();
    for index in 0..10 {
        let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
        let r = if index % 2 == 0 {
            radius
        } else {
            radius * 0.46
        };
        points.push(center + Vec2::angled(angle) * r);
    }
    if filled {
        painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
    } else {
        painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    }
}
