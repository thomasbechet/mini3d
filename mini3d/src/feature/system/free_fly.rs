use glam::{Quat, Vec3};

use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        instance::{ParallelResolver, SystemResult},
        query::Query,
    },
    feature::component::{common::free_fly::FreeFly, scene::transform::Transform},
    registry::{
        component::{ComponentData, StaticComponent},
        error::RegistryError,
        system::ParallelSystem,
    },
};

#[derive(Default)]
pub struct FreeFlySystem {
    free_fly: StaticComponent<FreeFly>,
    transform: StaticComponent<Transform>,
    query: Query,
}

impl ParallelSystem for FreeFlySystem {
    const NAME: &'static str = "free_fly_system";

    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.free_fly = resolver.read(FreeFly::UID)?;
        self.transform = resolver.write(Transform::UID)?;
        self.query = resolver
            .query()
            .all(&[FreeFly::UID, Transform::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
        let mut transforms = ecs.view_mut(self.transform)?;
        let mut free_flies = ecs.view_mut(self.free_fly)?;

        for e in ecs.query(self.query) {
            let transform = transforms.get_mut(e).unwrap();
            let free_fly = free_flies.get_mut(e).unwrap();

            // Check active
            if !free_fly.active {
                continue;
            }

            // Update view mod
            if api.input.action(free_fly.switch_mode)?.is_just_pressed() {
                free_fly.free_mode = !free_fly.free_mode;
            }

            // Compute camera translation
            let mut direction = Vec3::ZERO;
            direction += transform.forward() * api.input.axis(free_fly.move_forward)?.value;
            direction += transform.backward() * api.input.axis(free_fly.move_backward)?.value;
            direction += transform.left() * api.input.axis(free_fly.move_left)?.value;
            direction += transform.right() * api.input.axis(free_fly.move_right)?.value;
            if free_fly.free_mode {
                direction += transform.up() * api.input.axis(free_fly.move_up)?.value;
                direction += transform.down() * api.input.axis(free_fly.move_down)?.value;
            } else {
                direction += Vec3::Y * api.input.axis(free_fly.move_up)?.value;
                direction += Vec3::NEG_Y * api.input.axis(free_fly.move_down)?.value;
            }
            let direction_length = direction.length();
            direction = direction.normalize_or_zero();

            // Camera speed
            let mut speed = FreeFly::NORMAL_SPEED;
            if api.input.action(free_fly.move_fast)?.is_pressed() {
                speed = FreeFly::FAST_SPEED;
            } else if api.input.action(free_fly.move_slow)?.is_pressed() {
                speed = FreeFly::SLOW_SPEED;
            }

            // Apply transformation
            transform.translation += direction * direction_length * api.time.delta() as f32 * speed;

            // Apply rotation
            let motion_x = api.input.axis(free_fly.view_x)?.value;
            let motion_y = api.input.axis(free_fly.view_y)?.value;
            if free_fly.free_mode {
                if motion_x != 0.0 {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Y,
                        -f32::to_radians(motion_x)
                            * FreeFly::ROTATION_SENSIBILITY
                            * api.time.delta() as f32,
                    );
                }
                if motion_y != 0.0 {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::X,
                        f32::to_radians(motion_y)
                            * FreeFly::ROTATION_SENSIBILITY
                            * api.time.delta() as f32,
                    );
                }
                if api.input.action(free_fly.roll_left)?.is_pressed() {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Z,
                        -f32::to_radians(FreeFly::ROLL_SPEED) * api.time.delta() as f32,
                    );
                }
                if api.input.action(free_fly.roll_right)?.is_pressed() {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Z,
                        f32::to_radians(FreeFly::ROLL_SPEED) * api.time.delta() as f32,
                    );
                }
            } else {
                if motion_x != 0.0 {
                    free_fly.yaw +=
                        motion_x * FreeFly::ROTATION_SENSIBILITY * api.time.delta() as f32;
                }
                if motion_y != 0.0 {
                    free_fly.pitch +=
                        motion_y * FreeFly::ROTATION_SENSIBILITY * api.time.delta() as f32;
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
}
