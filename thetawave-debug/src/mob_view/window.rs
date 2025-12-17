use bevy::{
    camera::{RenderTarget, ScalingMode},
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::With,
        schedule::ScheduleLabel,
        system::{Commands, Query, ResMut},
    },
    prelude::{Camera, Camera2d, OrthographicProjection, Projection, Transform},
    utils::default,
    window::{Window, WindowClosed, WindowRef, WindowResolution},
};
use bevy_egui::EguiMultipassSchedule;
use bevy_enoki::{prelude::ColorParticle2dMaterial, ParticleSpawner};
use thetawave_core::ParticleRenderingEnabled;

use super::data::{MobViewCamera, MobViewWindow, MobViewWindowState};

/// Custom schedule for the mob view window's egui rendering
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MobViewContextPass;

/// Spawns the mob view window and camera
pub fn spawn_mob_view_window(
    mut commands: Commands,
    mut state: ResMut<MobViewWindowState>,
    mut particles: ResMut<ParticleRenderingEnabled>,
    particle_query: Query<Entity, With<ParticleSpawner<ColorParticle2dMaterial>>>,
) {
    if state.is_open {
        return;
    }

    // Disable particles to avoid bevy_enoki crash with multiple cameras
    particles.0 = false;

    // Despawn all existing particle entities to prevent render issues
    for entity in particle_query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn secondary window
    let window_entity = commands
        .spawn((
            Window {
                title: "Mob View".to_string(),
                resolution: WindowResolution::new(800, 600),
                resizable: true,
                ..Default::default()
            },
            MobViewWindow,
        ))
        .id();

    // Spawn camera targeting this window
    // The EguiMultipassSchedule goes on the camera, not the window
    const VIEWPORT_HEIGHT: f32 = 200.0;

    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                order: 2,
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: VIEWPORT_HEIGHT,
                },
                ..OrthographicProjection::default_2d()
            }),
            Transform::from_scale(bevy::math::Vec3::splat(state.zoom_level)),
            MobViewCamera,
            EguiMultipassSchedule::new(MobViewContextPass),
        ))
        .id();

    state.window_entity = Some(window_entity);
    state.camera_entity = Some(camera_entity);
    state.is_open = true;
}

/// Despawns the mob view window and camera
pub fn despawn_mob_view_window(
    mut commands: Commands,
    mut state: ResMut<MobViewWindowState>,
    mut particles: ResMut<ParticleRenderingEnabled>,
) {
    if let Some(window) = state.window_entity {
        commands.entity(window).despawn();
    }
    if let Some(camera) = state.camera_entity {
        commands.entity(camera).despawn();
    }

    state.window_entity = None;
    state.camera_entity = None;
    state.is_open = false;

    // Re-enable particles now that the second camera is gone
    particles.0 = true;
}

/// Handles window close events from the OS
pub fn handle_mob_view_window_close(
    mut commands: Commands,
    mut state: ResMut<MobViewWindowState>,
    mut particles: ResMut<ParticleRenderingEnabled>,
    mut window_closed_events: MessageReader<WindowClosed>,
) {
    for event in window_closed_events.read() {
        // Check if the mob view window was closed
        if Some(event.window) == state.window_entity {
            // Despawn the camera entity to clean up EguiMultipassSchedule
            if let Some(camera) = state.camera_entity {
                commands.entity(camera).despawn();
            }
            state.window_entity = None;
            state.camera_entity = None;
            state.is_open = false;
            // Re-enable particles now that the second camera is gone
            particles.0 = true;
        } else if state.is_open {
            // Some other window (likely the primary/main game window) was closed
            // Close the mob view window too
            if let Some(window) = state.window_entity {
                commands.entity(window).despawn();
            }
            if let Some(camera) = state.camera_entity {
                commands.entity(camera).despawn();
            }
            state.window_entity = None;
            state.camera_entity = None;
            state.is_open = false;
            particles.0 = true;
        }
    }
}

/// Marker for toggle mob view window message
#[derive(bevy::ecs::message::Message)]
pub struct ToggleMobViewWindowEvent;

/// Toggles the mob view window open/closed
pub fn toggle_mob_view_window(
    commands: Commands,
    state: ResMut<MobViewWindowState>,
    particles: ResMut<ParticleRenderingEnabled>,
    particle_query: Query<Entity, With<ParticleSpawner<ColorParticle2dMaterial>>>,
) {
    if state.is_open {
        despawn_mob_view_window(commands, state, particles);
    } else {
        spawn_mob_view_window(commands, state, particles, particle_query);
    }
}
