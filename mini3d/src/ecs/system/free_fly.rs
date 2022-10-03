use glam::{Vec3, Quat};
use hecs::World;

use crate::{ecs::component::{transform::TransformComponent, free_fly::FreeFlyComponent}, input::InputManager};

pub fn system_free_fly(world: &mut World, input: &InputManager, delta_time: f32) {
    for (_, (transform, free_view)) in world.query_mut::<(&mut TransformComponent, &mut FreeFlyComponent)>() {

        // Update view mod
        if input.action(free_view.switch_mode).map_or(false, |b| b.is_just_pressed()) {
            free_view.free_mode = !free_view.free_mode;
        }

        // Compute camera translation
        let mut direction = Vec3::ZERO;
        direction += transform.forward() * input.axis(free_view.move_forward).map_or(0.0, |a| a.value);
        direction += transform.backward() * input.axis(free_view.move_backward).map_or(0.0, |a| a.value);
        direction += transform.left() * input.axis(free_view.move_left).map_or(0.0, |a| a.value);
        direction += transform.right() * input.axis(free_view.move_right).map_or(0.0, |a| a.value);
        if free_view.free_mode {
            direction += transform.up() * input.axis(free_view.move_up).map_or(0.0, |a| a.value);
            direction += transform.down() * input.axis(free_view.move_down).map_or(0.0, |a| a.value);
        } else {
            direction += Vec3::Y * input.axis(free_view.move_up).map_or(0.0, |a| a.value);
            direction += Vec3::NEG_Y * input.axis(free_view.move_down).map_or(0.0, |a| a.value);
        }
        let direction_length = direction.length();
        direction = direction.normalize_or_zero();

        // Camera speed
        let speed = FreeFlyComponent::NORMAL_SPEED;
        //TODO: fast, slow input modes

        // Apply transformation
        transform.translation += direction * direction_length * delta_time * speed;
    
        // Apply rotation
        let motion_x = input.axis(free_view.look_x).map_or(0.0, |a| a.value);
        let motion_y = input.axis(free_view.look_y).map_or(0.0, |a| a.value);
        if free_view.free_mode {
            if motion_x != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::Y, -f32::to_radians(motion_x) * FreeFlyComponent::ROTATION_SENSIBILITY);
            }
            if motion_y != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(motion_y) * FreeFlyComponent::ROTATION_SENSIBILITY);
            }
            if input.action(free_view.roll_left).map_or(false, |b| b.is_pressed()) {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, -f32::to_radians(FreeFlyComponent::ROLL_SPEED) * delta_time);
            }
            if input.action(free_view.roll_right).map_or(false, |b| b.is_pressed()) {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, f32::to_radians(FreeFlyComponent::ROLL_SPEED) * delta_time);
            }
            
        } else {
            if motion_x != 0.0 {
                free_view.yaw += motion_x * FreeFlyComponent::ROTATION_SENSIBILITY;
            }
            if motion_y != 0.0 {
                free_view.pitch += motion_y * FreeFlyComponent::ROTATION_SENSIBILITY;
            }
        
            if free_view.pitch < -90.0 { free_view.pitch = -90.0 };
            if free_view.pitch > 90.0 { free_view.pitch = 90.0 };
        
            let mut rotation = Quat::from_axis_angle(Vec3::Y, -f32::to_radians(free_view.yaw));
            rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(free_view.pitch));
            transform.rotation = rotation;
        }        
    }
}