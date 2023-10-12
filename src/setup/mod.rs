use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use iyes_progress::prelude::AssetsLoading;
use rand::Rng;
use std::f32::consts::PI;

use crate::animation::ActiveAnimation;
use crate::{
    actions::{Action, JumpState},
    Animations, Faction,
};
use crate::{Cell, ControlledPlayer, Game};
use leafwing_input_manager::prelude::*;

const BOARD_SIZE_I: usize = 14;
const BOARD_SIZE_J: usize = 21;

#[derive(Resource)]
pub struct Assets3D(pub Handle<Gltf>);

pub fn load_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let gltf = asset_server.load("models/ivory/ivory.glb");
    loading.add(&gltf);
    commands.insert_resource(Assets3D(gltf));
}

pub fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        ..default()
    });

    // spawn the game board
    let cell_scene = asset_server.load("models/world/tile.glb#Scene0");
    game.board = (0..BOARD_SIZE_J)
        .map(|j| {
            (0..BOARD_SIZE_I)
                .map(|i| {
                    let height = rand::thread_rng().gen_range(-0.1..0.1);
                    let transform = Transform::from_xyz(i as f32, height, j as f32);
                    commands
                        .spawn(SceneBundle {
                            transform,
                            scene: cell_scene.clone(),
                            ..default()
                        })
                        .insert(RigidBody::Fixed)
                        .insert(Collider::cuboid(0.5, 0.2, 0.5));
                    Cell { height }
                })
                .collect()
        })
        .collect();
}

pub fn setup(
    mut commands: Commands,
    mut game: ResMut<Game>,
    models: Res<Assets3D>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    game.player.i = BOARD_SIZE_I / 2;
    game.player.j = BOARD_SIZE_J / 2;

    info!(
        "Animations: {:?}",
        assets_gltf.get(&models.0).unwrap().named_animations.keys()
    );

    // named_scenes
    // named_meshes

    info!(
        "Scenes: {:?}",
        assets_gltf.get(&models.0).unwrap().named_scenes.keys()
    );

    info!(
        "Meshes: {:?}",
        assets_gltf.get(&models.0).unwrap().named_meshes.keys()
    );

    let named_animations = &assets_gltf.get(&models.0).unwrap().named_animations;

    let animations = Animations {
        idle: named_animations["ivory_idle"].clone(),
        run: named_animations["ivory_run"].clone(),
        attack: named_animations["ivory_slash"].clone(),
    };

    let active_animation = ActiveAnimation::new(animations);

    // spawn the game character
    let entity = commands
        .spawn_empty()
        .insert(SceneBundle {
            transform: Transform {
                translation: Vec3::new(
                    game.player.i as f32,
                    game.board[game.player.j][game.player.i].height + 1.0,
                    game.player.j as f32,
                ),
                rotation: Quat::from_rotation_y(-PI / 2.),
                ..default()
            },
            scene: assets_gltf.get(&models.0).unwrap().named_scenes["Run"].clone(),
            ..default()
        })
        .insert(active_animation)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(0.20),
            ..default()
        })
        .insert(JumpState::from(30.))
        .insert(Faction(0))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule(
            Vec3::new(0.0, 0.28, 0.0),
            Vec3::new(0.0, 1.4, 0.0),
            0.26,
        ))
        .insert(ControlledPlayer)
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::Space, Action::Jump),
                (KeyCode::W, Action::MoveNorth),
                (KeyCode::A, Action::MoveWest),
                (KeyCode::S, Action::MoveSouth),
                (KeyCode::D, Action::MoveEast),
            ]),
        })
        .id();
    game.player.entity = Some(entity);
}

// remove all entities that are not a camera or window
pub fn teardown(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
