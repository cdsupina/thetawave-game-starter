use bevy::ecs::event::Event;

/// Event for updating the cam
#[derive(Event)]
pub struct CameraZoomEvent(pub i8);
