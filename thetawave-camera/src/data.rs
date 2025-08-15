use bevy::ecs::event::Event;

/// Event for updating the 2d camera zoom level
#[derive(Event)]
pub struct Camera2DZoomEvent(pub i8);

/// Event for updating the 3d camera zoom level
#[derive(Event)]
pub struct Camera3DZoomEvent(pub i8);
