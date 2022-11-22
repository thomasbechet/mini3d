use anyhow::Result;
use glam::{Vec3, Quat};
use hecs::World;

use crate::{ecs::SystemContext, feature::component::{transform::TransformComponent, free_fly::FreeFlyComponent}};

pub fn run(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    for (_, (transform, free_fly)) in world.query_mut::<(&mut TransformComponent, &mut FreeFlyComponent)>() {

        // Check active
        if !free_fly.active { continue; }

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
        let speed = FreeFlyComponent::NORMAL_SPEED;
        //TODO: fast, slow input modes

        // Apply transformation
        transform.translation += direction * direction_length * ctx.delta_time as f32 * speed;
    
        // Apply rotation
        let motion_x = ctx.input.axis(free_fly.view_x)?.value;
        let motion_y = ctx.input.axis(free_fly.view_y)?.value;
        if free_fly.free_mode {
            if motion_x != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::Y, -f32::to_radians(motion_x) * FreeFlyComponent::ROTATION_SENSIBILITY * ctx.delta_time as f32);
            }
            if motion_y != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(motion_y) * FreeFlyComponent::ROTATION_SENSIBILITY * ctx.delta_time as f32);
            }
            if ctx.input.action(free_fly.roll_left)?.is_pressed() {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, -f32::to_radians(FreeFlyComponent::ROLL_SPEED) * ctx.delta_time as f32);
            }
            if ctx.input.action(free_fly.roll_right)?.is_pressed() {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, f32::to_radians(FreeFlyComponent::ROLL_SPEED) * ctx.delta_time as f32);
            }
            
        } else {
            if motion_x != 0.0 {
                free_fly.yaw += motion_x * FreeFlyComponent::ROTATION_SENSIBILITY * ctx.delta_time as f32;
            }
            if motion_y != 0.0 {
                free_fly.pitch += motion_y * FreeFlyComponent::ROTATION_SENSIBILITY * ctx.delta_time as f32;
            }
        
            if free_fly.pitch < -90.0 { free_fly.pitch = -90.0 };
            if free_fly.pitch > 90.0 { free_fly.pitch = 90.0 };
        
            let mut rotation = Quat::from_axis_angle(Vec3::Y, -f32::to_radians(free_fly.yaw));
            rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(free_fly.pitch));
            transform.rotation = rotation;
        }
    }

    Ok(())
}