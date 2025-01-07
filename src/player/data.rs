use bevy::prelude::Component;

#[derive(Component)]
pub(super) struct PlayerStatsComponent {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}
