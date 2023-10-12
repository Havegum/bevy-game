// Let's outline the steps we'll need to take to implement the agent.
// We're writing it in Bevy.
// For a start, let's consider these states:
// - wandering
// - chasing
// - attacking
//
// We will implement the agent as a utility AI.

use crate::{actions::JumpState, Faction, GameState};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::actions::ActionState as BBActionState;
use big_brain::prelude::*;
use leafwing_input_manager::{
    action_state::{ActionData, ActionState},
    axislike::DualAxisData,
    InputManagerBundle,
};

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Wanderlust;

#[derive(Default, Debug, Clone, Component, ActionBuilder)]
pub struct Wander {
    pub target: Option<Vec3>,
    pub timer: Option<f32>,
}

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Alert;

#[derive(Default, Debug, Clone, Component, ActionBuilder)]
pub struct Chase;

#[derive(Default, Debug, Clone, Component)]
pub struct Chaser {
    pub target: Option<Entity>,
}

#[derive(Default)]
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate))
            .add_systems(
                PreUpdate,
                (wanderlust_scorer_system, alert_scorer_system)
                    .in_set(BigBrainSet::Scorers)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (wandering_action_system, chase_action_system)
                    .in_set(BigBrainSet::Actions)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_mob(mut commands: Commands, asset_server: Res<AssetServer>) {
    let thinker = Thinker::build()
        .picker(Highest)
        .when(Alert, Chase)
        .otherwise(Wander::default());

    commands
        .spawn_empty()
        .insert(SceneBundle {
            transform: Transform {
                translation: Vec3::new(5., 1., 7.),
                ..default()
            },
            scene: asset_server.load("models/AlienCake/enemy.glb#Scene0"),
            ..default()
        })
        .insert(InputManagerBundle::<crate::actions::Action>::default())
        .insert(KinematicCharacterController::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(Faction(1))
        .insert(Collider::capsule(
            Vec3::new(0.0, 0.25, 0.0),
            Vec3::new(0.0, 0.8, 0.0),
            0.25,
        ))
        .insert(JumpState::from(30.0))
        .insert(Chaser::default())
        .insert(thinker);
}

pub fn wanderlust_scorer_system(mut query: Query<&mut Score, With<Wanderlust>>) {
    for mut score in &mut query {
        score.set(0.1);
    }
}

pub fn wandering_action_system(mut query: Query<(&Actor, &mut BBActionState), With<Wander>>) {
    for (Actor(_actor), mut state) in &mut query {
        match *state {
            BBActionState::Requested => {
                // TODO: set target, timer, and move there
                *state = BBActionState::Success;
            }
            BBActionState::Cancelled => {
                *state = BBActionState::Failure;
            }
            _ => {}
        }
    }
    // }
}

pub fn chase_action_system(
    transform: Query<&Transform>,
    mut action_states: Query<(
        &mut ActionState<crate::actions::Action>,
        &Chaser,
        &Transform,
    )>,
    mut query: Query<(&Actor, &mut BBActionState), With<Chase>>,
) {
    for (Actor(actor), mut state) in &mut query {
        if let Ok((mut action_state, chaser, self_transform)) = action_states.get_mut(*actor) {
            let Some(target) = chaser.target else {
                continue;
            };

            let Ok(target_transform) = transform.get(target) else {
                continue;
            };

            let vector = (target_transform.translation - self_transform.translation)
                .normalize()
                .xz();

            match *state {
                BBActionState::Requested => {
                    action_state.set_action_data(
                        crate::actions::Action::Move,
                        ActionData {
                            state: leafwing_input_manager::buttonlike::ButtonState::Pressed,
                            value: 1.,
                            timing: leafwing_input_manager::action_state::Timing::default(),
                            consumed: false,
                            axis_pair: Some(DualAxisData::from_xy(vector)),
                        },
                    );
                    // TODO: set target, timer, and move there, _then_ set success
                    // *state = BBActionState::Success;
                }
                BBActionState::Cancelled => {
                    *state = BBActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

pub fn alert_scorer_system(
    mut chaser: Query<&mut Chaser>,
    mut query: Query<(&Actor, &mut Score), With<Alert>>,
    transforms: Query<(Entity, &Transform, &Faction)>,
) {
    let chase_threshold = 8.0;

    for (Actor(actor), mut score) in &mut query {
        let mut nearest_distance = f32::MAX;
        let mut nearest_entity = None;

        let chaser = chaser.get_mut(*actor);

        if let Ok(chaser) = &chaser {
            if let Some(target) = chaser.target {
                if let Ok((_, transform, _)) = transforms.get(target) {
                    nearest_distance = transform.translation.distance(Vec3::ZERO);
                    nearest_entity = Some(target);
                    if nearest_distance < chase_threshold {
                        continue;
                    }
                }
            }
        }

        if let Ok((_, self_transform, Faction(self_faction))) = transforms.get(*actor) {
            for (entity, transform, _) in transforms
                .iter()
                .filter(|(_, _, Faction(faction))| faction != self_faction)
                .filter(|(entity, _, _)| entity != actor)
            {
                let distance = transform.translation.distance(self_transform.translation);
                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest_entity = Some(entity);
                }
            }
        }

        if let Ok(mut chaser) = chaser {
            chaser.target = nearest_entity;
        }

        if nearest_entity.is_some() && nearest_distance < 8.0 {
            score.set(1.0);
        } else {
            score.set(0.0);
        };
    }
}

//  TODO: MeleeAttacker
// "MeleeAttacker" is generic over the type of attack.
// It has a target, and a cooldown.
