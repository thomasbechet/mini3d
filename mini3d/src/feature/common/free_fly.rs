use mini3d_derive::{fixed, Component, Reflect, Serialize};

use crate::{
    api::{input::Input, time::Time, Context},
    ecs::{
        error::ResolverError,
        query::Query,
        system::{ParallelSystem, SystemResolver},
        view::native::single::NativeSingleViewMut,
    },
    expect,
    feature::input::{action::InputActionHandle, axis::InputAxisHandle},
    math::{
        fixed::{FixedPoint, TrigFixedPoint, I32F16, U32F16},
        quat::Q,
        vec::V3,
    },
};

use super::transform::Transform;

#[derive(Default, Component, Reflect, Clone, Serialize)]
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
    pub yaw: I32F16,
    pub pitch: I32F16,
}

impl FreeFly {
    pub const NORMAL_SPEED: U32F16 = fixed!(10);
    pub const FAST_SPEED: U32F16 = fixed!(25);
    pub const SLOW_SPEED: U32F16 = fixed!(3);
    pub const ROLL_SPEED: U32F16 = fixed!(60);
    pub const ROTATION_SENSIBILITY: U32F16 = fixed!(180);
    pub const ZOOM_SPEED: U32F16 = fixed!(10);
}

#[derive(Default, Clone)]
pub struct FreeFlySystem {
    free_fly: NativeSingleViewMut<FreeFly>,
    transform: NativeSingleViewMut<Transform>,
    query: Query,
}

impl FreeFlySystem {
    pub const NAME: &'static str = "SYS_FreeFly";
}

impl ParallelSystem for FreeFlySystem {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.free_fly.resolve(resolver, FreeFly::NAME)?;
        self.transform.resolve(resolver, Transform::NAME)?;
        self.query
            .resolve(resolver)
            .all(&[FreeFly::NAME, Transform::NAME])?;
        Ok(())
    }

    fn run(mut self, ctx: &Context) {
        for e in self.query.iter() {
            let transform = &mut self.transform[e];
            let free_fly = &mut self.free_fly[e];

            // Check active
            if !free_fly.active {
                continue;
            }

            // Update view mod
            if expect!(ctx, Input::action(ctx, free_fly.switch_mode)).is_just_pressed() {
                free_fly.free_mode = !free_fly.free_mode;
            }

            // Compute camera translation
            let mut direction = V3::ZERO;
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
                direction += V3::Y * expect!(ctx, Input::axis(ctx, free_fly.move_up)).value;
                direction += V3::NEG_Y * expect!(ctx, Input::axis(ctx, free_fly.move_down)).value;
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
            transform.translation +=
                direction * direction_length * (Time::delta(ctx) * speed).convert();

            // Apply rotation
            let motion_x = expect!(ctx, Input::axis(ctx, free_fly.view_x)).value;
            let motion_y = expect!(ctx, Input::axis(ctx, free_fly.view_y)).value;
            if free_fly.free_mode {
                if motion_x != fixed!(0) {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Y,
                        -motion_x.to_radians()
                            * (FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)).convert(),
                    );
                }
                if motion_y != fixed!(0) {
                    transform.rotation *= Q::from_axis_angle(
                        V3::X,
                        motion_y.to_radians()
                            * (FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)).convert(),
                    );
                }
                if expect!(ctx, Input::action(ctx, free_fly.roll_left)).is_pressed() {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Z,
                        -(FreeFly::ROLL_SPEED.to_radians() * Time::delta(ctx)).convert::<I32F16>(),
                    );
                }
                if expect!(ctx, Input::action(ctx, free_fly.roll_right)).is_pressed() {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Z,
                        (FreeFly::ROLL_SPEED.to_radians() * Time::delta(ctx)).convert(),
                    );
                }
            } else {
                if motion_x != fixed!(0) {
                    free_fly.yaw +=
                        motion_x * (FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)).convert();
                }
                if motion_y != fixed!(0) {
                    free_fly.pitch +=
                        motion_y * (FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)).convert();
                }

                if free_fly.pitch < fixed!(-90.0) {
                    free_fly.pitch = fixed!(-90.0)
                };
                if free_fly.pitch > fixed!(90.0) {
                    free_fly.pitch = fixed!(90.0)
                };

                let mut rotation = Q::from_axis_angle(V3::Y, -free_fly.yaw.to_radians());
                rotation *= Q::from_axis_angle(V3::X, free_fly.pitch.to_radians());
                transform.rotation = rotation;
            }
        }
    }
}
