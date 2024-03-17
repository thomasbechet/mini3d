use mini3d_derive::{fixed, Component, Reflect, Serialize};

use crate::{
    ecs::{
        context::{Context, Time},
        entity::Entity,
        query::Query,
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    input::component::{InputAction, InputAxis},
    math::{
        fixed::{FixedPoint, TrigFixedPoint, I32F16, U32F16},
        quat::Q,
        vec::V3,
    },
};

use super::{transform::Transform, SystemConfig, SystemParam};

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

pub const FREE_FLY_SYSTEM_CONFIG: SystemConfig = SystemConfig::new(&[
    SystemParam::ViewMut(FreeFly::NAME),
    SystemParam::ViewMut(Transform::NAME),
    SystemParam::ViewRef(InputAction::NAME),
    SystemParam::ViewRef(InputAxis::NAME),
    SystemParam::Query {
        all: &[FreeFly::NAME, Transform::NAME],
        any: &[],
        not: &[],
    },
]);

fn freefly_system(
    ctx: &Context,
    mut v_free_fly: NativeSingleViewMut<FreeFly>,
    mut v_transform: NativeSingleViewMut<Transform>,
    v_input_action: NativeSingleViewRef<InputAction>,
    v_input_axis: NativeSingleViewRef<InputAxis>,
    query: Query,
) {
    for e in query.iter() {
        let transform = &mut v_transform[e];
        let free_fly = &mut v_free_fly[e];

        // Check active
        if !free_fly.active {
            continue;
        }

        // Update view mode
        if v_input_action[free_fly.switch_mode].is_just_pressed() {
            free_fly.free_mode = !free_fly.free_mode;
        }

        // Compute camera translation
        let mut direction = V3::ZERO;
        direction += transform.forward() * v_input_axis[free_fly.move_forward].value();
        direction += transform.backward() * v_input_axis[free_fly.move_backward].value();
        direction += transform.left() * v_input_axis[free_fly.move_left].value();
        direction += transform.right() * v_input_axis[free_fly.move_right].value();
        if free_fly.free_mode {
            direction += transform.up() * v_input_axis[free_fly.move_up].value();
            direction += transform.down() * v_input_axis[free_fly.move_down].value();
        } else {
            direction += V3::Y * v_input_axis[free_fly.move_up].value();
            direction += V3::NEG_Y * v_input_axis[free_fly.move_down].value();
        }
        let direction_length = direction.length();
        direction = direction.normalize_or_zero();

        // Camera speed
        let mut speed = FreeFly::NORMAL_SPEED;
        if v_input_action[free_fly.move_fast].is_pressed() {
            speed = FreeFly::FAST_SPEED;
        } else if v_input_action[free_fly.move_slow].is_pressed() {
            speed = FreeFly::SLOW_SPEED;
        }

        // Apply transformation
        transform.translation +=
            direction * direction_length * I32F16::cast(Time::delta(ctx) * speed);

        // Apply rotation
        let motion_x = v_input_axis[free_fly.view_x].value();
        let motion_y = v_input_axis[free_fly.view_y].value();
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
            if v_input_action[free_fly.roll_left].is_pressed() {
                transform.rotation *= Q::from_axis_angle(
                    V3::Z,
                    -I32F16::cast(FreeFly::ROLL_SPEED.to_radians() * Time::delta(ctx)),
                );
            }
            if v_input_action[free_fly.roll_right].is_pressed() {
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
