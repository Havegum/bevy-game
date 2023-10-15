use crate::actions::Action;
use crate::{Game, GameState};
use bevy::input::mouse::MouseButtonInput;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::action_state::ActionData;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::ActionState;

pub fn cursor_system(
    camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    transform: Query<&Transform>,
    game: ResMut<Game>,
    mut gizmos: Gizmos,
) {
    // If there's no player, just return early
    let Some(entity) = game.player.entity else {
        return;
    };

    let Ok(transform) = transform.get(entity) else {
        return;
    };

    let (camera, camera_transform) = camera.single();

    // If there's no cursor return early
    let Some(position) = q_windows.single().cursor_position() else {
        return;
    };

    // If the cursor doesn't point anywhere, return early
    if let Some(point) = camera.viewport_to_world(camera_transform, position) {
        let t = -point.origin.y / point.direction.y;
        let intersection = point.origin + t * point.direction + Vec3::Y * 0.1;
        gizmos.sphere(intersection, Quat::IDENTITY, 0.25, Color::RED);

        let mut intersection = intersection;
        intersection.y = transform.translation.y;
    }
}

pub fn mouse_button_events(
    camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut player_query: Query<(&mut ActionState<Action>, &Transform)>,

    game: ResMut<Game>,
) {
    use bevy::input::ButtonState;

    for ev in &mut mousebtn_evr {
        let Some(player) = game.player.entity else {
            return;
        };

        let Ok((mut action_state, transform)) = player_query.get_mut(player) else {
            return;
        };

        let (camera, camera_transform) = camera.single();
        if ev.state == ButtonState::Pressed {
            let Some(position) = q_windows.single().cursor_position() else {
                continue;
            };

            if let Some(point) = camera.viewport_to_world(camera_transform, position) {
                let t = -point.origin.y / point.direction.y;
                let intersection = point.origin + t * point.direction + Vec3::Y * 0.1;

                let mut intersection = intersection;
                intersection.y = transform.translation.y;

                let vector = intersection.xz() - transform.translation.xz();

                action_state.set_action_data(
                    Action::Attack,
                    ActionData {
                        state: leafwing_input_manager::buttonlike::ButtonState::Pressed,
                        value: 1.,
                        axis_pair: Some(DualAxisData::from_xy(vector)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

// restart the game when pressing spacebar
pub fn gameover_keyboard(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
