use std::f32::consts::TAU;


use bevy::prelude::Vec2;
use bevy_prototype_lyon::prelude::PathBuilder;

pub fn draw_arrow(path: &mut PathBuilder,  radius: f32, start_angle: f32, sweep_angle: f32) {

    let centre = Vec2::ZERO; //todo remove
    let path_start = centre + point_at_angle(radius, start_angle);
    let path_end = centre + point_at_angle(radius, start_angle + sweep_angle);

    const ARROW_WIDTH: f32 = 6.0;
    const ARROW_LENGTH: f32 = 100.0;
    let arrow_angle = ARROW_LENGTH * sweep_angle.signum() / (radius * TAU);
    if sweep_angle.abs() > arrow_angle.abs() {
        path.move_to(path_start);

        path.arc(
            centre,
            Vec2 {
                x: radius,
                y: radius,
            },
            sweep_angle - arrow_angle,
            0.0,
        );

        let arrow_point = centre + point_at_angle(radius, start_angle + sweep_angle - arrow_angle);

        path.move_to(arrow_point); // just incase

        path.line_to(arrow_point.lerp(centre, ARROW_WIDTH / radius));
        path.line_to(path_end);

        path.line_to(arrow_point.lerp(centre, -ARROW_WIDTH / radius));

        path.line_to(arrow_point);
    }
}


fn point_at_angle(dist: f32, radians: f32) -> Vec2 {
    let x = dist * (radians).cos();
    let y = dist * (radians).sin();
    Vec2 { x, y }
}