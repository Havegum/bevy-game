use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

#[derive(Bundle)]
pub struct HealthBar {
    pub health: Health,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
