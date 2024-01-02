use mini3d_derive::{fixed, Component, Reflect, Serialize};

use crate::{
    ecs::{
        context::{Context, Time},
        entity::Entity,
        error::ResolverError,
        query::Query,
        system::{ParallelSystem, SystemResolver},
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    input::component::{InputAction, InputAxis},
    math::{
        fixed::{FixedPoint, TrigFixedPoint, I32F16, U32F16},
        quat::Q,
        vec::V3,
    },
};

use super::transform::Transform;

#[derive(Default, Component, Reflect, Clone, Serialize)]
#[component(storage = "single")]
pub struct FreeFly {
    // Control if free fly is active
    pub active: bool,

    // Inputs
    pub switch_mode: Entity,
    pub roll_left: Entity,
    pub roll_right: Entity,
    pub view_x: Entity,
    pub view_y: Entity,
    pub move_forward: Entity,
    pub move_backward: Entity,
    pub move_up: Entity,
    pub move_down: Entity,
    pub move_left: Entity,
    pub move_right: Entity,
    pub move_fast: Entity,
    pub move_slow: Entity,

    // View data
    pub free_mode: bool,
    pub yaw: I32F16,
    pub pitch: I32F16,
}

impl FreeFly {
    pub const NORMAL_SPEED: U32F16 = U32F16::from_int(10);
    pub const FAST_SPEED: U32F16 = U32F16::from_int(25);
    pub const SLOW_SPEED: U32F16 = U32F16::from_int(3);
    pub const ROLL_SPEED: U32F16 = U32F16::from_int(60);
    pub const ROTATION_SENSIBILITY: U32F16 = U32F16::from_int(180);
    pub const ZOOM_SPEED: U32F16 = U32F16::from_int(10);
}

#[derive(Default, Clone)]
pub struct FreeFlySystem {
    free_fly: NativeSingleViewMut<FreeFly>,
    transform: NativeSingleViewMut<Transform>,
    input_action: NativeSingleViewRef<InputAction>,
    input_axis: NativeSingleViewRef<InputAxis>,
    query: Query,
}

impl FreeFlySystem {
    pub const NAME: &'static str = "SYS_FreeFly";
}

fn freefly_system(
    ctx: &Context,
    free_fly: &mut NativeSingleView<FreeFly>,
    transform: &mut NativeSingleView<Transform>,
    input_action: &mut NativeSingleView<InputAction>,
    input_axis: &mut NativeSingleView<InputAxis>,
    query: &Query,
) -> Result<(), ResolverError> {
    Ok(())
}

impl ParallelSystem for FreeFlySystem {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.free_fly.resolve(resolver, FreeFly::NAME)?;
        self.transform.resolve(resolver, Transform::NAME)?;
        self.input_action.resolve(resolver, InputAction::NAME)?;
        self.input_axis.resolve(resolver, InputAction::NAME)?;
        self.query
            .resolve(resolver)
            .all(&[FreeFly::NAME, Transform::NAME])?;
        Ok(())
    }

    fn run(&mut self, ctx: &Context) {
        for e in self.query.iter() {
            let transform = &mut self.transform[e];
            let free_fly = &mut self.free_fly[e];

            // Check active
            if !free_fly.active {
                continue;
            }

            // Update view mode
            if self.input_action[free_fly.switch_mode].is_just_pressed() {
                free_fly.free_mode = !free_fly.free_mode;
            }

            // Compute camera translation
            let mut direction = V3::ZERO;
            direction += transform.forward() * self.input_axis[free_fly.move_forward].value();
            direction += transform.backward() * self.input_axis[free_fly.move_backward].value();
            direction += transform.left() * self.input_axis[free_fly.move_left].value();
            direction += transform.right() * self.input_axis[free_fly.move_right].value();
            if free_fly.free_mode {
                direction += transform.up() * self.input_axis[free_fly.move_up].value();
                direction += transform.down() * self.input_axis[free_fly.move_down].value();
            } else {
                direction += V3::Y * self.input_axis[free_fly.move_up].value();
                direction += V3::NEG_Y * self.input_axis[free_fly.move_down].value();
            }
            let direction_length = direction.length();
            direction = direction.normalize_or_zero();

            // Camera speed
            let mut speed = FreeFly::NORMAL_SPEED;
            if self.input_action[free_fly.move_fast].is_pressed() {
                speed = FreeFly::FAST_SPEED;
            } else if self.input_action[free_fly.move_slow].is_pressed() {
                speed = FreeFly::SLOW_SPEED;
            }

            // Apply transformation
            transform.translation +=
                direction * direction_length * I32F16::cast(Time::delta(ctx) * speed);

            // Apply rotation
            let motion_x = self.input_axis[free_fly.view_x].value();
            let motion_y = self.input_axis[free_fly.view_y].value();
            if free_fly.free_mode {
                if motion_x != fixed!(0) {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Y,
                        -motion_x.to_radians()
                            * I32F16::cast(FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)),
                    );
                }
                if motion_y != fixed!(0) {
                    transform.rotation *= Q::from_axis_angle(
                        V3::X,
                        motion_y.to_radians()
                            * I32F16::cast(FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx)),
                    );
                }
                if self.input_action[free_fly.roll_left].is_pressed() {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Z,
                        -I32F16::cast(FreeFly::ROLL_SPEED.to_radians() * Time::delta(ctx)),
                    );
                }
                if self.input_action[free_fly.roll_right].is_pressed() {
                    transform.rotation *= Q::from_axis_angle(
                        V3::Z,
                        I32F16::cast(FreeFly::ROLL_SPEED.to_radians() * Time::delta(ctx)),
                    );
                }
            } else {
                if motion_x != fixed!(0) {
                    free_fly.yaw +=
                        motion_x * I32F16::cast(FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx));
                }
                if motion_y != fixed!(0) {
                    free_fly.pitch +=
                        motion_y * I32F16::cast(FreeFly::ROTATION_SENSIBILITY * Time::delta(ctx));
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
