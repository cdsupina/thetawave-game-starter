use bevy::ecs::message::Message;

/// Message for updating the 2d camera zoom level
#[derive(Message)]
pub struct Camera2DZoomEvent(pub i8);

/// Message for updating the 3d camera zoom level
#[derive(Message)]
pub struct Camera3DZoomEvent(pub i8);
