use std::time::Duration;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Query, Res},
    },
    prelude::Commands,
    time::{Time, Timer, TimerMode},
};

pub trait ConditionTrait {
    fn tick(&mut self, delta: Duration);
    fn stack(&self) -> usize;
    fn add_timer(&mut self, duration: Duration);
}

/// Base trait for all conditions.
/// Conditions are components that indicate that an entity is in a certain state.
/// This trait counts the number of times the condition is applied.
///
/// It also keeps track of the timers associated with each effect source.
/// When the timer expires, the effect is removed.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Condition<T> {
    stack: usize,
    timers: Vec<Timer>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Condition<T> {
    /// Create a new condition.
    pub fn new(duration: Duration) -> Self {
        Self {
            stack: 1,
            timers: vec![Timer::new(duration, TimerMode::Once)],
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ConditionTrait for Condition<T> {
    /// Add a timer to the condition.
    fn add_timer(&mut self, duration: Duration) {
        self.stack += 1;
        self.timers.push(Timer::new(duration, TimerMode::Once));
    }

    fn stack(&self) -> usize {
        self.stack
    }

    fn tick(&mut self, delta: Duration) {
        for timer in &mut self.timers {
            timer.tick(delta);
            if timer.just_finished() {
                self.stack -= 1;
            }
        }
    }
}

pub fn conditions_system<Condition>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Condition)>,
    time: Res<Time>,
) where
    Condition: ConditionTrait + Component,
{
    for (entity, mut condition) in &mut query {
        condition.tick(time.delta());

        if condition.stack() == 0 {
            commands.entity(entity).remove::<Condition>();
        }
    }
}

pub struct Locked;

// Se: https://docs.rs/bevy/latest/bevy/ecs/query/trait.WorldQuery.html#adding-methods-to-query-items
