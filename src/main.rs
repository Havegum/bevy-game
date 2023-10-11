#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    // clippy::nursery,
    // clippy::cargo
  )]
#![allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::type_complexity
)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use iyes_progress::prelude::*;
use leafwing_input_manager::prelude::*;

mod actions;
use actions::{
    conditions::{Condition, *},
    *,
};

mod camera;
use camera::*;

mod input;
use input::*;

mod agent;
use agent::*;

mod animation;
use animation::*;

mod setup;
use setup::*;

mod utils;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    LoadingGame,
    Playing,
    GameOver,
}

#[derive(Component)]
pub struct Faction(u32);

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_plugins(MobPlugin)
        .add_plugins(
            ProgressPlugin::new(GameState::LoadingGame)
                .continue_to(GameState::Playing)
                .track_assets(),
        )
        .insert_resource(RapierConfiguration::default())
        .init_resource::<Game>()
        .add_systems(OnEnter(GameState::LoadingGame), load_gltf)
        .add_systems(Startup, setup_cameras)
        .add_systems(
            OnEnter(GameState::Playing),
            (setup, setup_scene /*spawn_mob*/),
        )
        .add_systems(
            Update,
            (
                animate_upon_load,
                animate_upon_change,
                move_system,
                focus_camera,
                mouse_button_events.before(attack_system),
                attack_system,
                jump,
                cursor_system,
                conditions_system::<Condition<Locked>>,
                ActiveAnimation::queue_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), teardown)
        .add_systems(
            Update,
            (
                gameover_keyboard.run_if(in_state(GameState::GameOver)),
                bevy::window::close_on_esc,
            ),
        )
        .add_systems(OnExit(GameState::GameOver), teardown)
        .run();
}

struct Cell {
    height: f32,
}

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    i: usize,
    j: usize,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Resource, Default)]
pub struct Game {
    board: Vec<Vec<Cell>>,
    player: Player,
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
}
