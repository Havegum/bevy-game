use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::{f32::consts::PI, time::Duration};

use crate::animation::{ActiveAnimation, Animations};

pub mod conditions;
use conditions::{Condition, Locked};

#[derive(Actionlike, PartialEq, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Jump,
    Move,
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    Attack,
}

#[derive(Default, Component)]
pub struct JumpState {
    available: bool,
    strength: f32,
    i: f32,
}

impl From<f32> for JumpState {
    fn from(strength: f32) -> JumpState {
        JumpState {
            available: false,
            strength,
            i: 0.0,
        }
    }
}

// impl JumpPlugin {
//     fn system()
// }

// impl Plugin for JumpPlugin {
//     fn build(&self, app: &mut AppBuilder) {
//         app.add_system(jump.system());
//     }
// }

pub fn jump(
    mut transforms: Query<
        (
            &ActionState<Action>,
            &mut KinematicCharacterController,
            &mut JumpState,
            &KinematicCharacterControllerOutput,
        ),
        Without<Condition<Locked>>,
    >,
    time_step: Res<Time>,
) {
    let gravity: f32 = 9.81 * time_step.delta_seconds();

    for (action_state, mut transform, mut jump_state, output) in &mut transforms {
        let mut gravity = gravity;
        let mut translation = transform.translation.unwrap_or_default();

        if jump_state.i > 0.0 {
            jump_state.i -= time_step.delta_seconds();
        }

        let pressed_jump = action_state.pressed(Action::Jump);

        if pressed_jump {
            gravity *= 0.8;
            if jump_state.available {
                jump_state.i = 1.0;
                jump_state.available = false;
            }
        }

        if output.grounded {
            jump_state.available = true;
        }

        if jump_state.i > 0.0 {
            let jump_boost = if pressed_jump { 1.0 } else { 0.6 };

            let jump_strength =
                jump_state.i.powi(6) * jump_state.strength * time_step.delta_seconds() * jump_boost;
            translation.y += jump_strength;
        }

        translation.y -= gravity;
        transform.translation = Some(translation);
    }
}

// TODO: Move ActionState, Kin.Cha.Con., and Transform to a struct that impls WorldQuery.
// Do the same with ActiveAnimation and Animations.
//
// See: https://docs.rs/bevy/latest/bevy/ecs/query/trait.WorldQuery.html#adding-methods-to-query-items
pub fn move_system(
    mut agent_query: Query<
        (
            &ActionState<Action>,
            &mut KinematicCharacterController,
            &mut Transform,
            Option<&mut ActiveAnimation>,
        ),
        Without<Condition<Locked>>,
    >,
    time_step: Res<Time>,
) {
    let delta = time_step.delta_seconds() * 10.;
    let speed = 0.5 * delta;

    for (action_state, mut controller, mut transform, active_animation) in &mut agent_query {
        let mut translation_delta = Vec3::ZERO;

        let rotation = Quat::from_rotation_y(-PI * 1.75);
        if action_state.pressed(Action::MoveNorth) {
            translation_delta += rotation * Vec3::X;
        }

        if action_state.pressed(Action::MoveSouth) {
            translation_delta -= rotation * Vec3::X;
        }

        if action_state.pressed(Action::MoveEast) {
            translation_delta += rotation * Vec3::Z;
        }

        if action_state.pressed(Action::MoveWest) {
            translation_delta -= rotation * Vec3::Z;
        }

        if action_state.pressed(Action::Move) {
            let action = action_state.action_data(Action::Move);
            if let Some(axis_pair) = action.axis_pair {
                translation_delta += Vec3::new(axis_pair.x(), 0., axis_pair.y());
            }
        }

        translation_delta = translation_delta.normalize_or_zero() * speed;

        let mut translation = controller.translation.unwrap_or_default();
        translation += translation_delta;
        controller.translation = Some(translation);

        let is_moving = translation_delta.xz().length() > 0.0;

        if is_moving {
            let mut direction = translation_delta;
            direction.y = 0.0;
            let new_rotation = crate::utils::look_to(direction);
            transform.rotation = transform.rotation.slerp(new_rotation, 0.4);
        }

        if let Some(mut active_animation) = active_animation {
            let animation = active_animation.get();
            if is_moving {
                if animation != active_animation.animations.run {
                    let animation = active_animation.animations.run.clone_weak();
                    active_animation.set(animation);
                }
            } else if animation == active_animation.animations.run {
                let animation = active_animation.animations.idle.clone_weak();
                active_animation.set(animation);
            }
        }
    }
}

pub fn attack_system(
    mut commands: Commands,
    mut agent_query: Query<(
        Entity,
        &ActionState<Action>,
        &mut KinematicCharacterController,
        &mut Transform,
        Option<&mut ActiveAnimation>,
    )>,
    time_step: Res<Time>,
) {
    for (entity, action_state, mut controller, mut transform, active_animation) in &mut agent_query
    {
        if action_state.pressed(Action::Attack) {
            if let Some(mut active_animation) = active_animation {
                let attack = active_animation.animations.attack.clone_weak();
                let idle = active_animation.animations.idle.clone_weak();
                active_animation.set(attack).then(idle);

                commands
                    .entity(entity)
                    .insert(Condition::<Locked>::new(Duration::from_millis(600)));
            }
        }
    }
}
