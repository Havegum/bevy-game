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
    pub animations: Animations,
    active: Handle<AnimationClip>,
    transition: Duration,
    repeating: bool,
    next: Option<Handle<AnimationClip>>,
    timer: Option<Timer>,
    changed: bool,
}

impl ActiveAnimation {
    pub fn new(animations: Animations) -> Self {
        Self {
            active: animations.idle.clone_weak(),
            animations,
            transition: Duration::from_millis(250),
            repeating: true,
            next: None,
            timer: None,
            changed: false,
        }
    }

    pub fn get(&self) -> Handle<AnimationClip> {
        self.active.clone_weak()
    }

    pub fn next(&mut self) -> &mut Self {
        let next = self.next.take().unwrap();
        self.set(next);
        self
    }

    pub fn transition_duration(&self) -> Duration {
        self.transition
    }

    pub fn set(&mut self, animation: Handle<AnimationClip>) -> &mut Self {
        self.active = animation;
        self.repeating = true;
        self.changed = true;
        self
    }

    pub fn then(&mut self, animation: Handle<AnimationClip>) -> &mut Self {
        self.next = Some(animation);
        self.repeating = false;
        self
    }

    pub fn queue_system(
        mut animation_query: Query<(&mut ActiveAnimation, &Children)>,
        children_query: Query<&Children>,
        animations: Res<Assets<AnimationClip>>,
        mut player_query: Query<&mut AnimationPlayer>,
        time: Res<Time>,
    ) {
        for (mut animation, children) in &mut animation_query {
            if animation.next.is_some() {
                if let Some(timer) = animation.timer.as_mut() {
                    timer.tick(time.delta());

                    if timer.finished() {
                        animation.timer = None;
                        animation.next();
                    }
                } else if let Some(clip) = animations.get(&animation.get()) {
                    animation.timer = Some(Timer::from_seconds(clip.duration(), TimerMode::Once));
                }
            } else if !animation.changed {
                continue;
            } else {
                animation.changed = false;
            }

            for child in children {
                for child in children_query.get(*child).unwrap() {
                    if let Ok(mut player) = player_query.get_mut(*child) {
                        if animation.transition > Duration::ZERO {
                            player.play_with_transition(
                                animation.get(),
                                animation.transition_duration(),
                            );
                        } else {
                            player.play(animation.get());
                        }

                        if animation.repeating {
                            player.repeat();
                        } else {
                            player.stop_repeating();
                        }

                        break;
                    }
                }
            }
        }
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

// pub fn animate_upon_change(
//     animation_query: Query<(&ActiveAnimation, &Children), Changed<ActiveAnimation>>,
//     children_query: Query<&Children>,
//     mut player_query: Query<&mut AnimationPlayer>,
// ) {
//     for (animation, children) in animation_query.iter() {
//         for child in children {
//             for child in children_query.get(*child).unwrap() {
//                 if let Ok(mut player) = player_query.get_mut(*child) {
//                     if animation.transition > Duration::ZERO {
//                         player
//                             .play_with_transition(animation.get(), animation.transition_duration());
//                     } else {
//                         player.play(animation.get());
//                     }

//                     if animation.repeating {
//                         player.repeat();
//                     } else {
//                         player.stop_repeating();
//                     }

//                     break;
//                 }
//             }
//         }
//     }
// }
