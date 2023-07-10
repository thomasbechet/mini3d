use glam::{Quat, Vec3};

use crate::{
    context::SystemContext,
    ecs::system::SystemResult,
    feature::component::{free_fly::FreeFly, transform::Transform},
    registry::component::Component,
};

pub fn run(ctx: &mut SystemContext) -> SystemResult {
    let world = ctx.world.active();
    let mut transforms = world.static_view_mut::<Transform>(Transform::UID)?;
    let mut free_flies = world.static_view_mut::<FreeFly>(FreeFly::UID)?;

    for e in &world.query(&[Transform::UID, FreeFly::UID]) {
        let transform = transforms.get_mut(e).unwrap();
        let free_fly = free_flies.get_mut(e).unwrap();

        // Check active
        if !free_fly.active {
            continue;
        }

        // Update view mod
        if ctx.input.action(free_fly.switch_mode)?.is_just_pressed() {
            free_fly.free_mode = !free_fly.free_mode;
        }

        // Compute camera translation
        let mut direction = Vec3::ZERO;
        direction += transform.forward() * ctx.input.axis(free_fly.move_forward)?.value;
        direction += transform.backward() * ctx.input.axis(free_fly.move_backward)?.value;
        direction += transform.left() * ctx.input.axis(free_fly.move_left)?.value;
        direction += transform.right() * ctx.input.axis(free_fly.move_right)?.value;
        if free_fly.free_mode {
            direction += transform.up() * ctx.input.axis(free_fly.move_up)?.value;
            direction += transform.down() * ctx.input.axis(free_fly.move_down)?.value;
        } else {
            direction += Vec3::Y * ctx.input.axis(free_fly.move_up)?.value;
            direction += Vec3::NEG_Y * ctx.input.axis(free_fly.move_down)?.value;
        }
        let direction_length = direction.length();
        direction = direction.normalize_or_zero();

        // Camera speed
        let mut speed = FreeFly::NORMAL_SPEED;
        if ctx.input.action(free_fly.move_fast)?.is_pressed() {
            speed = FreeFly::FAST_SPEED;
        } else if ctx.input.action(free_fly.move_slow)?.is_pressed() {
            speed = FreeFly::SLOW_SPEED;
        }

        // Apply transformation
        transform.translation += direction * direction_length * ctx.time.delta() as f32 * speed;

        // Apply rotation
        let motion_x = ctx.input.axis(free_fly.view_x)?.value;
        let motion_y = ctx.input.axis(free_fly.view_y)?.value;
        if free_fly.free_mode {
            if motion_x != 0.0 {
                transform.rotation *= Quat::from_axis_angle(
                    Vec3::Y,
                    -f32::to_radians(motion_x)
                        * FreeFly::ROTATION_SENSIBILITY
                        * ctx.time.delta() as f32,
                );
            }
            if motion_y != 0.0 {
                transform.rotation *= Quat::from_axis_angle(
                    Vec3::X,
                    f32::to_radians(motion_y)
                        * FreeFly::ROTATION_SENSIBILITY
                        * ctx.time.delta() as f32,
                );
            }
            if ctx.input.action(free_fly.roll_left)?.is_pressed() {
                transform.rotation *= Quat::from_axis_angle(
                    Vec3::Z,
                    -f32::to_radians(FreeFly::ROLL_SPEED) * ctx.time.delta() as f32,
                );
            }
            if ctx.input.action(free_fly.roll_right)?.is_pressed() {
                transform.rotation *= Quat::from_axis_angle(
                    Vec3::Z,
                    f32::to_radians(FreeFly::ROLL_SPEED) * ctx.time.delta() as f32,
                );
            }
        } else {
            if motion_x != 0.0 {
                free_fly.yaw += motion_x * FreeFly::ROTATION_SENSIBILITY * ctx.time.delta() as f32;
            }
            if motion_y != 0.0 {
                free_fly.pitch +=
                    motion_y * FreeFly::ROTATION_SENSIBILITY * ctx.time.delta() as f32;
            }

            if free_fly.pitch < -90.0 {
                free_fly.pitch = -90.0
            };
            if free_fly.pitch > 90.0 {
                free_fly.pitch = 90.0
            };

            let mut rotation = Quat::from_axis_angle(Vec3::Y, -f32::to_radians(free_fly.yaw));
            rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(free_fly.pitch));
            transform.rotation = rotation;
        }
    }

    Ok(())
}
