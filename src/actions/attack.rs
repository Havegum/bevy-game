//! This will drive most of the combat system
//!
//! Every attack will spawn a new entity with the "Attack" component.
//! Attacks have the following properties. They might be organized
//! as fields, or as components. Components are probably best.
//!
//! * Effect
//! * Hitbox
//! * `HitSet` // a Set of entities that have been hit
//! * Duration
//! * Velocity
//! * Faction
//! * Animation / model / effect / sound
//!
//! Attacks are short-lived entities. As long as they live, they
//! will check for collisions with other entities that are of an
//! enemy faction. If they collide
//! with an entity, they will apply damage and effects to that entity.
//! Then, that entity will be added to the "Hit" set so that it won't
//! apply itself again.
//!
//! Meelee attacks have a short duration and zero velocity.
//!
//! Projectile attacks have a velocity and no duration. They will be
//! destroyed when they have collided `n` times.
//!
//!
//! --
//!
//! Effects contain all the information about how an attack will affect
//! an entity. They are applied to entities that are hit by an attack.
//! Effects have the following properties. Or maybe they, too, should
//! be components.
//!
//! * Damage
//! * Conditions
//! * Knockback
//! * Animation / model / effect / sound
//!
//! --
//!
//! Conditions is a Vector of conditions that are applied to hit entities.

use crate::{ConditionType, Faction};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use core::time::Duration;
use std::collections::HashSet;

#[derive(Component, Clone, Default)]
pub struct LifeSpan {
    pub timer: Timer,
    // pub anticipation: Duration,
    pub attack: Duration,
}

impl LifeSpan {
    pub fn new(attack: Duration) -> Self {
        Self {
            timer: Timer::new(attack, TimerMode::Once),
            // anticipation,
            attack,
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }

    pub fn system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut LifeSpan)>,
        time: Res<Time>,
    ) {
        for (entity, mut lifespan) in &mut query {
            lifespan.tick(time.delta());
            if lifespan.timer.just_finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct Attack {
    pub effect: Effect,
    pub hit_set: HashSet<Entity>,
    pub velocity: Option<Velocity>,
    pub faction: Faction,
}

#[derive(Bundle, Clone, Default)]
pub struct AttackBundle {
    pub attack: Attack,
    pub hit_box: Collider,
    pub lifespan: LifeSpan,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    sensor: Sensor,
}

#[derive(Component, Clone, Default)]
pub struct PendingAttack {
    pub bundle: AttackBundle,
    pub timer: Timer,
}

impl PendingAttack {
    pub fn new(bundle: AttackBundle, delay: Duration) -> Self {
        Self {
            bundle,
            timer: Timer::new(delay, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }

    pub fn system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut PendingAttack)>,
        time: Res<Time>,
    ) {
        for (entity, mut pending_attack) in &mut query {
            pending_attack.tick(time.delta());
            if pending_attack.timer.just_finished() {
                commands.spawn(std::mem::take(&mut pending_attack.bundle));
                commands
                    .get_entity(entity)
                    .unwrap()
                    .remove::<PendingAttack>();
            }
        }
    }
}

impl AttackBundle {
    pub fn new(
        effect: Effect,
        hit_box: Collider,
        faction: Faction,
        attack_duration: Duration,
        transform: Transform,
    ) -> Self {
        Self {
            attack: Attack {
                effect,
                hit_set: HashSet::new(),
                velocity: None,
                faction,
            },
            hit_box,
            sensor: Sensor,
            lifespan: LifeSpan::new(attack_duration),
            transform,
            global_transform: GlobalTransform::default(),
        }
    }
}

#[derive(Component, Clone)]
pub struct Effect {
    pub damage: f32,
    pub conditions: Vec<ConditionType>,
    pub knockback: Vec3,
}

impl Effect {
    pub fn new(damage: f32, conditions: Vec<ConditionType>, knockback: Vec3) -> Self {
        Self {
            damage,
            conditions,
            knockback,
        }
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self {
            damage: 0.0,
            conditions: vec![],
            knockback: Vec3::ZERO,
        }
    }
}
