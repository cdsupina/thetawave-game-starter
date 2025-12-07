//! Local implementation of cooldown system, replacing leafwing_abilities.
//!
//! Provides `Cooldown` for individual ability timers and `CooldownState<A>`
//! for managing cooldowns across multiple abilities.

use bevy::{ecs::component::Component, platform::collections::HashMap};
use core::time::Duration;
use std::hash::Hash;

/// Error returned when attempting to trigger an ability that is on cooldown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CooldownNotReady;

/// A cooldown timer for a single ability.
///
/// Tracks the remaining time until the ability is ready, and the total duration
/// of the cooldown period.
#[derive(Debug, Clone)]
pub struct Cooldown {
    /// Time remaining until the ability is ready (0.0 = ready)
    remaining: f32,
    /// Total cooldown duration in seconds
    duration: f32,
}

impl Cooldown {
    /// Creates a new cooldown with the specified duration in seconds.
    /// The cooldown starts in the ready state (remaining = 0).
    pub fn from_secs(secs: f32) -> Self {
        Self {
            remaining: 0.0,
            duration: secs,
        }
    }

    /// Advances the cooldown timer by the given duration.
    pub fn tick(&mut self, delta: Duration) {
        self.remaining = (self.remaining - delta.as_secs_f32()).max(0.0);
    }

    /// Returns true if the cooldown is ready (not on cooldown).
    pub fn ready(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Attempts to trigger the ability, starting the cooldown.
    ///
    /// Returns `Ok(())` if the ability was ready and is now on cooldown.
    /// Returns `Err(CooldownNotReady)` if the ability is still on cooldown.
    pub fn trigger(&mut self) -> Result<(), CooldownNotReady> {
        if self.ready() {
            self.remaining = self.duration;
            Ok(())
        } else {
            Err(CooldownNotReady)
        }
    }

    /// Resets the cooldown to the ready state immediately.
    /// Used for duration-based abilities that need to stay ready during their effect.
    pub fn refresh(&mut self) {
        self.remaining = 0.0;
    }

    /// Returns the remaining cooldown time in seconds.
    pub fn remaining(&self) -> f32 {
        self.remaining
    }

    /// Returns the total cooldown duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Returns the cooldown progress as a value from 0.0 (just triggered) to 1.0 (ready).
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            1.0 - (self.remaining / self.duration)
        }
    }
}

/// Component that manages cooldown states for multiple abilities.
///
/// Generic over the ability type `A`, which must be hashable and comparable.
#[derive(Component, Debug, Clone)]
pub struct CooldownState<A: Hash + Eq> {
    cooldowns: HashMap<A, Cooldown>,
}

impl<A: Hash + Eq> CooldownState<A> {
    /// Creates a new `CooldownState` from an iterator of (ability, cooldown) pairs.
    pub fn new(pairs: impl IntoIterator<Item = (A, Cooldown)>) -> Self {
        Self {
            cooldowns: pairs.into_iter().collect(),
        }
    }

    /// Attempts to trigger the specified ability.
    ///
    /// Returns `Ok(())` if the ability was ready and is now on cooldown.
    /// Returns `Err(CooldownNotReady)` if the ability is on cooldown or not found.
    pub fn trigger(&mut self, ability: &A) -> Result<(), CooldownNotReady> {
        self.cooldowns
            .get_mut(ability)
            .ok_or(CooldownNotReady)?
            .trigger()
    }

    /// Returns a mutable reference to the cooldown for the specified ability.
    pub fn get_mut(&mut self, ability: &A) -> Option<&mut Cooldown> {
        self.cooldowns.get_mut(ability)
    }

    /// Returns an immutable reference to the cooldown for the specified ability.
    pub fn get(&self, ability: &A) -> Option<&Cooldown> {
        self.cooldowns.get(ability)
    }

    /// Advances all cooldown timers by the given duration.
    pub fn tick_all(&mut self, delta: Duration) {
        for cooldown in self.cooldowns.values_mut() {
            cooldown.tick(delta);
        }
    }

    /// Returns true if the specified ability is ready (not on cooldown).
    pub fn ready(&self, ability: &A) -> bool {
        self.cooldowns
            .get(ability)
            .map(|c| c.ready())
            .unwrap_or(false)
    }
}

impl<A: Hash + Eq> Default for CooldownState<A> {
    fn default() -> Self {
        Self {
            cooldowns: HashMap::default(),
        }
    }
}
