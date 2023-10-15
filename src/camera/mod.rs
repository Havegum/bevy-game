use crate::Game;
use bevy::prelude::*;

pub fn setup_cameras(mut commands: Commands, mut game: ResMut<Game>) {
    game.camera_should_focus = Vec3::ZERO;
    game.camera_is_focus = game.camera_should_focus;
    commands.spawn(Camera3dBundle {
        transform: Transform::IDENTITY.looking_at(game.camera_is_focus, Vec3::Y),
        projection: Projection::Orthographic(OrthographicProjection {
            scale: 0.01,
            ..default()
        }),
        ..default()
    });
}

/// change the focus of the camera
pub fn focus_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut transforms: ParamSet<(Query<&mut Transform, With<Camera3d>>, Query<&Transform>)>,
) {
    const SPEED: f32 = 5.0;

    if let Some(player_entity) = game.player.entity {
        if let Ok(player_transform) = transforms.p1().get(player_entity) {
            game.camera_should_focus = player_transform.translation;
        }
    } else {
        game.camera_should_focus = Vec3::ZERO;
    }

    let mut camera_motion = game.camera_should_focus - game.camera_is_focus;
    if camera_motion.length() > 0.2 {
        camera_motion *= SPEED * time.delta_seconds();
        // set the new camera's actual focus
        game.camera_is_focus += camera_motion;
    }
    // look at that new camera's actual focus
    for mut transform in transforms.p0().iter_mut() {
        transform.translation = Vec3::new(
            game.camera_is_focus.x - 5.0,
            game.camera_is_focus.y + 4.0,
            game.camera_is_focus.z + 5.0,
        );
        *transform = transform.looking_at(game.camera_is_focus, Vec3::Y);
    }
}
