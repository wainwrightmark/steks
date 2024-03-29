use bevy::prelude::Vec2;

/// Adds a sub-path from a polygon but rounds the corners.
///
/// There must be no sub-path in progress when this method is called.
/// No sub-path is in progress after the method is called.
pub fn make_rounded_polygon_path(points: &[Vec2], radius: f32) -> String {
    if points.len() < 2 {
        return "".to_string();
    }

    //p points are original polygon points
    //q points are the actual points we will draw lines and arcs between
    let clamped_radius = clamp_radius(radius, points[points.len() - 1], points[0], points[1]);
    let q_first = get_point_between(points[0], points[1], clamped_radius);

    //We begin on the line just after the first point
    let mut builder = format!("M{} {}", q_first.x, q_first.y);
    //builder.begin(q_first, attributes);

    for index in 0..points.len() {
        let p_current = points[index];
        let p_next = points[(index + 1) % points.len()];
        let p_after_next = points[(index + 2) % points.len()];

        let clamped_radius = clamp_radius(radius, p_current, p_next, p_after_next);

        //q1 is the second point on the line between p_current and p_next
        let q1 = get_point_between(p_next, p_current, clamped_radius);
        //q2 is the first point on the line between p_next and p_after_next
        let q2 = get_point_between(p_next, p_after_next, clamped_radius);

        line_to(&mut builder, q1);
        let turn_winding = get_winding(p_current, p_next, p_after_next);

        //Draw the arc near p_next
        arc(
            &mut builder,
            (clamped_radius, clamped_radius),
            0.0,
            ArcFlags {
                large_arc: false,
                sweep: turn_winding,
            },
            q2,
        );
    }

    builder.push('z');

    builder
}

#[derive(Debug, Copy, Clone)]
struct ArcFlags {
    pub large_arc: bool,
    pub sweep: Winding,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Winding {
    Positive,
    Negative,
}

fn clamp_radius(radius: f32, p_previous: Vec2, p_current: Vec2, p_next: Vec2) -> f32 {
    let shorter_edge = ((p_current - p_next).length()).min((p_previous - p_current).length());
    radius.min(shorter_edge * 0.5)
}

fn get_point_between(p1: Vec2, p2: Vec2, radius: f32) -> Vec2 {
    let dist = p1.distance(p2);
    let ratio = radius / dist;

    p1.lerp(p2, ratio)
}

fn get_winding(p0: Vec2, p1: Vec2, p2: Vec2) -> Winding {
    let cross = cross_product(p2 - p0, p1 - p0);
    if cross.is_sign_positive() {
        Winding::Positive
    } else {
        Winding::Negative
    }
}

#[inline]
pub fn cross_product(first: Vec2, second: Vec2) -> f32 {
    first.x * second.y - first.y * second.x
}

fn line_to(builder: &mut String, to: Vec2) {
    builder.push_str(format!("L {} {}", to.x, to.y).as_str())
}

fn arc(builder: &mut String, radii: (f32, f32), x_rotation: f32, flags: ArcFlags, to: Vec2) {
    let rx = radii.0;
    let ry = radii.1;
    let large_arc = flags.large_arc.then_some(1).unwrap_or_default();
    let sweep = if flags.sweep == Winding::Negative {
        1
    } else {
        0
    };
    let x = to.x;
    let y = to.y;

    builder.push_str(format!("A {rx} {ry} {x_rotation} {large_arc} {sweep} {x} {y}").as_str())
}
