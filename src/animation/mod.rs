use std::time::Duration;

use bevy::prelude::*;

#[derive(Component)]
pub struct Animations {
    pub run: Handle<AnimationClip>,
    pub idle: Handle<AnimationClip>,
    pub attack: Handle<AnimationClip>,
}

#[derive(Component)]
pub struct ActiveAnimation {
    animation: Handle<AnimationClip>,
    transition: Duration,
    repeating: bool,
    next: Option<Handle<AnimationClip>>,
}

impl ActiveAnimation {
    pub fn new(animation: Handle<AnimationClip>) -> Self {
        Self {
            animation,
            transition: Duration::from_millis(250),
            repeating: true,
            next: None,
        }
    }

    pub fn get(&self) -> Handle<AnimationClip> {
        self.animation.clone_weak()
    }

    pub fn transition_duration(&self) -> Duration {
        self.transition
    }

    pub fn set(&mut self, animation: Handle<AnimationClip>) -> &mut Self {
        self.animation = animation;
        self
    }

    pub fn then(&mut self, animation: Handle<AnimationClip>) -> &mut Self {
        self.next = Some(animation);
        self.repeating = false;
        self
    }

    pub fn with_duration(&mut self, duration: Duration) -> &mut Self {
        self.transition = duration;
        self
    }

    pub fn queue_system(mut query: Query<&mut ActiveAnimation>) {
        // TODO: implement this.
        // I'll probably move out this bit to like a "Action"-component
        // Then that component can queue up the next action — idle, probably – as well
        // as handling whether the action is cancelable or not.
        // for mut active_animation in &mut query {
        //     if active_animation.next.is_some() {
        //         let next = active_animation.next.take().unwrap();
        //         active_animation.animation = next;
        //         active_animation.repeating = true;
        //     }
        // }
    }
}

pub fn animate_upon_load(
    animation_query: Query<&ActiveAnimation>,
    parent_query: Query<&Parent>,
    mut player_query: Query<(&mut AnimationPlayer, &Parent), Added<AnimationPlayer>>,
) {
    for (mut player, parent) in &mut player_query {
        let mut parent = parent.get();

        while let Ok(parent_parent) = parent_query.get(parent) {
            if let Ok(animation) = animation_query.get(parent_parent.get()) {
                player.play(animation.get()).repeat();
                break;
            }
            parent = parent_parent.get();
        }
    }
}

pub fn animate_upon_change(
    animation_query: Query<(&ActiveAnimation, &Children), Changed<ActiveAnimation>>,
    children_query: Query<&Children>,
    mut player_query: Query<&mut AnimationPlayer>,
) {
    for (animation, children) in animation_query.iter() {
        for child in children {
            for child in children_query.get(*child).unwrap() {
                if let Ok(mut player) = player_query.get_mut(*child) {
                    player
                        .play_with_transition(animation.get(), Duration::from_millis(20))
                        .repeat();
                    break;
                }
            }
        }
    }
}
