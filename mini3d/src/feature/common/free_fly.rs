use glam::{Quat, Vec3};
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        api::{ecs::ECS, input::Input, time::Time, Context},
        error::ResolverError,
        query::QueryId,
        system::{ParallelResolver, ParallelSystem},
    },
    expect,
    feature::core::component::ComponentId,
    input::handle::{InputActionHandle, InputAxisHandle},
};

use super::transform::Transform;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct FreeFly {
    // Control if free fly is active
    pub active: bool,

    // Inputs
    pub switch_mode: InputActionHandle,
    pub roll_left: InputActionHandle,
    pub roll_right: InputActionHandle,
    pub view_x: InputAxisHandle,
    pub view_y: InputAxisHandle,
    pub move_forward: InputAxisHandle,
    pub move_backward: InputAxisHandle,
    pub move_up: InputAxisHandle,
    pub move_down: InputAxisHandle,
    pub move_left: InputAxisHandle,
    pub move_right: InputAxisHandle,
    pub move_fast: InputActionHandle,
    pub move_slow: InputActionHandle,

    // View data
    pub free_mode: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl FreeFly {
    pub const NORMAL_SPEED: f32 = 10.0;
    pub const FAST_SPEED: f32 = 25.0;
    pub const SLOW_SPEED: f32 = 3.0;
    pub const ROLL_SPEED: f32 = 60.0;
    pub const ROTATION_SENSIBILITY: f32 = 180.0;
    pub const ZOOM_SPEED: f32 = 10.0;
}

#[derive(Default)]
pub struct FreeFlySystem {
    free_fly: ComponentId,
    transform: ComponentId,
    query: QueryId,
}

impl FreeFlySystem {
    pub const NAME: &'static str = "free_fly_system";
}

impl ParallelSystem for FreeFlySystem {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), ResolverError> {
        self.free_fly = resolver.read(FreeFly::NAME)?;
        self.transform = resolver.write(Transform::NAME)?;
        self.query = resolver
            .query()
            .all(&[FreeFly::NAME, Transform::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ctx: &Context) {
        let mut transforms = ECS::view_mut(ctx, self.transform);
        let mut free_flies = ECS::view_mut(ctx, self.free_fly);

        for e in ECS::query(ctx, self.query) {
            let transform = transforms.get_mut(e).unwrap();
            let free_fly = free_flies.get_mut(e).unwrap();

            // Check active
            if !free_fly.active {
                continue;
            }

            // Update view mod
            if expect!(ctx, Input::action(ctx, free_fly.switch_mode)).is_just_pressed() {
                free_fly.free_mode = !free_fly.free_mode;
            }

            // Compute camera translation
            let mut direction = Vec3::ZERO;
            direction +=
                transform.forward() * expect!(ctx, Input::axis(ctx, free_fly.move_forward)).value;
            direction +=
                transform.backward() * expect!(ctx, Input::axis(ctx, free_fly.move_backward)).value;
            direction +=
                transform.left() * expect!(ctx, Input::axis(ctx, free_fly.move_left)).value;
            direction +=
                transform.right() * expect!(ctx, Input::axis(ctx, free_fly.move_right)).value;
            if free_fly.free_mode {
                direction +=
                    transform.up() * expect!(ctx, Input::axis(ctx, free_fly.move_up)).value;
                direction +=
                    transform.down() * expect!(ctx, Input::axis(ctx, free_fly.move_down)).value;
            } else {
                direction += Vec3::Y * expect!(ctx, Input::axis(ctx, free_fly.move_up)).value;
                direction += Vec3::NEG_Y * expect!(ctx, Input::axis(ctx, free_fly.move_down)).value;
            }
            let direction_length = direction.length();
            direction = direction.normalize_or_zero();

            // Camera speed
            let mut speed = FreeFly::NORMAL_SPEED;
            if expect!(ctx, Input::action(ctx, free_fly.move_fast)).is_pressed() {
                speed = FreeFly::FAST_SPEED;
            } else if expect!(ctx, Input::action(ctx, free_fly.move_slow)).is_pressed() {
                speed = FreeFly::SLOW_SPEED;
            }

            // Apply transformation
            transform.translation += direction * direction_length * Time::delta(ctx) as f32 * speed;

            // Apply rotation
            let motion_x = expect!(ctx, Input::axis(ctx, free_fly.view_x)).value;
            let motion_y = expect!(ctx, Input::axis(ctx, free_fly.view_y)).value;
            if free_fly.free_mode {
                if motion_x != 0.0 {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Y,
                        -f32::to_radians(motion_x)
                            * FreeFly::ROTATION_SENSIBILITY
                            * Time::delta(ctx) as f32,
                    );
                }
                if motion_y != 0.0 {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::X,
                        f32::to_radians(motion_y)
                            * FreeFly::ROTATION_SENSIBILITY
                            * Time::delta(ctx) as f32,
                    );
                }
                if expect!(ctx, Input::action(ctx, free_fly.roll_left)).is_pressed() {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Z,
                        -f32::to_radians(FreeFly::ROLL_SPEED) * Time::delta(ctx) as f32,
                    );
                }
                if expect!(ctx, Input::action(ctx, free_fly.roll_right)).is_pressed() {
                    transform.rotation *= Quat::from_axis_angle(
                        Vec3::Z,
                        f32::to_radians(FreeFly::ROLL_SPEED) * Time::delta(ctx) as f32,
                    );
                }
            } else {
                if motion_x != 0.0 {
                    free_fly.yaw +=
                        motion_x * FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx) as f32;
                }
                if motion_y != 0.0 {
                    free_fly.pitch +=
                        motion_y * FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx) as f32;
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
    }
}
