use glam::{Vec3, Quat};
use hecs::World;

use crate::{ecs::component::{transform::TransformComponent, free_fly::FreeFlyComponent}, input::InputManager, asset::AssetManager};

pub fn system_free_fly(world: &mut World, input: &InputManager, asset: &AssetManager, delta_time: f32) {
    for (_, (transform, free_view)) in world.query_mut::<(&mut TransformComponent, &mut FreeFlyComponent)>() {

        // Update view mod
        if free_view.switch_mode.state(asset, input, false).is_just_pressed() {
            free_view.free_mode = !free_view.free_mode;
        }

        // Compute camera translation
        let mut direction = Vec3::ZERO;
        direction += transform.forward() * free_view.move_forward.state(asset, input, 0.0).value;
        direction += transform.backward() * free_view.move_backward.state(asset, input, 0.0).value;
        direction += transform.left() * free_view.move_left.state(asset, input, 0.0).value;
        direction += transform.right() * free_view.move_right.state(asset, input, 0.0).value;
        if free_view.free_mode {
            direction += transform.up() * free_view.move_up.state(asset, input, 0.0).value;
            direction += transform.down() * free_view.move_down.state(asset, input, 0.0).value;
        } else {
            direction += Vec3::Y * free_view.move_up.state(asset, input, 0.0).value;
            direction += Vec3::NEG_Y * free_view.move_down.state(asset, input, 0.0).value;
        }
        let direction_length = direction.length();
        direction = direction.normalize_or_zero();

        // Camera speed
        let speed = FreeFlyComponent::NORMAL_SPEED;
        //TODO: fast, slow input modes

        // Apply transformation
        transform.translation += direction * direction_length * delta_time * speed;
    
        // Apply rotation
        let motion_x = free_view.view_x.state(asset, input, 0.0).value;
        let motion_y = free_view.view_y.state(asset, input, 0.0).value;
        if free_view.free_mode {
            if motion_x != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::Y, -f32::to_radians(motion_x) * FreeFlyComponent::ROTATION_SENSIBILITY * delta_time);
            }
            if motion_y != 0.0 {
                transform.rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(motion_y) * FreeFlyComponent::ROTATION_SENSIBILITY * delta_time);
            }
            if free_view.roll_left.state(asset, input, false).is_pressed() {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, -f32::to_radians(FreeFlyComponent::ROLL_SPEED) * delta_time);
            }
            if free_view.roll_right.state(asset, input, false).is_pressed() {
                transform.rotation *= Quat::from_axis_angle(Vec3::Z, f32::to_radians(FreeFlyComponent::ROLL_SPEED) * delta_time);
            }
            
        } else {
            if motion_x != 0.0 {
                free_view.yaw += motion_x * FreeFlyComponent::ROTATION_SENSIBILITY * delta_time;
            }
            if motion_y != 0.0 {
                free_view.pitch += motion_y * FreeFlyComponent::ROTATION_SENSIBILITY * delta_time;
            }
        
            if free_view.pitch < -90.0 { free_view.pitch = -90.0 };
            if free_view.pitch > 90.0 { free_view.pitch = 90.0 };
        
            let mut rotation = Quat::from_axis_angle(Vec3::Y, -f32::to_radians(free_view.yaw));
            rotation *= Quat::from_axis_angle(Vec3::X, f32::to_radians(free_view.pitch));
            transform.rotation = rotation;
        }        
    }
}