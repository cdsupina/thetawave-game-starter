use bevy::{
    ecs::{
        message::{Message, MessageReader, MessageWriter},
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
};

use super::data::{MobGroupRegistry, MobViewWindowState};

/// Message for cycling mob selection
#[derive(Message)]
pub struct CycleMobSelectionEvent;

/// System that detects Tab presses and emits cycle events
pub fn tab_cycle_mob_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<MobViewWindowState>,
    mut cycle_event: MessageWriter<CycleMobSelectionEvent>,
) {
    // Only process Tab when mob view is open
    if state.is_open && keyboard.just_pressed(KeyCode::Tab) {
        cycle_event.write(CycleMobSelectionEvent);
    }
}

/// System that handles the cycle event and updates the registry
pub fn handle_cycle_mob_selection(
    mut cycle_events: MessageReader<CycleMobSelectionEvent>,
    mut registry: ResMut<MobGroupRegistry>,
) {
    for _ in cycle_events.read() {
        registry.cycle_next();
    }
}
