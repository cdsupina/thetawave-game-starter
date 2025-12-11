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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Cooldown Tests ====================

    #[test]
    fn new_cooldown_starts_ready() {
        let cooldown = Cooldown::from_secs(2.0);
        assert!(cooldown.ready());
        assert_eq!(cooldown.remaining(), 0.0);
    }

    #[test]
    fn cooldown_reports_correct_duration() {
        let cooldown = Cooldown::from_secs(5.0);
        assert_eq!(cooldown.duration(), 5.0);
    }

    #[test]
    fn trigger_starts_cooldown() {
        let mut cooldown = Cooldown::from_secs(1.0);
        assert!(cooldown.trigger().is_ok());
        assert!(!cooldown.ready());
        assert_eq!(cooldown.remaining(), 1.0);
    }

    #[test]
    fn trigger_fails_when_on_cooldown() {
        let mut cooldown = Cooldown::from_secs(1.0);
        cooldown.trigger().unwrap();
        assert_eq!(cooldown.trigger(), Err(CooldownNotReady));
    }

    #[test]
    fn tick_reduces_remaining_time() {
        let mut cooldown = Cooldown::from_secs(1.0);
        cooldown.trigger().unwrap();
        cooldown.tick(Duration::from_secs_f32(0.3));
        assert!((cooldown.remaining() - 0.7).abs() < 0.001);
    }

    #[test]
    fn tick_does_not_go_negative() {
        let mut cooldown = Cooldown::from_secs(1.0);
        cooldown.trigger().unwrap();
        cooldown.tick(Duration::from_secs_f32(5.0)); // Way over
        assert_eq!(cooldown.remaining(), 0.0);
        assert!(cooldown.ready());
    }

    #[test]
    fn tick_on_ready_cooldown_stays_at_zero() {
        let mut cooldown = Cooldown::from_secs(1.0);
        cooldown.tick(Duration::from_secs_f32(0.5));
        assert_eq!(cooldown.remaining(), 0.0);
        assert!(cooldown.ready());
    }

    #[test]
    fn progress_is_1_when_ready() {
        let cooldown = Cooldown::from_secs(1.0);
        assert_eq!(cooldown.progress(), 1.0);
    }

    #[test]
    fn progress_is_0_when_just_triggered() {
        let mut cooldown = Cooldown::from_secs(1.0);
        cooldown.trigger().unwrap();
        assert_eq!(cooldown.progress(), 0.0);
    }

    #[test]
    fn progress_is_half_when_halfway() {
        let mut cooldown = Cooldown::from_secs(2.0);
        cooldown.trigger().unwrap();
        cooldown.tick(Duration::from_secs_f32(1.0));
        assert!((cooldown.progress() - 0.5).abs() < 0.001);
    }

    #[test]
    fn zero_duration_cooldown_is_always_ready() {
        let mut cooldown = Cooldown::from_secs(0.0);
        // Should be ready initially
        assert!(cooldown.ready());
        // Trigger sets remaining to 0.0 (the duration)
        cooldown.trigger().unwrap();
        // Should still be ready since duration is 0
        assert!(cooldown.ready());
        // Progress should be 1.0 (handles division by zero case)
        assert_eq!(cooldown.progress(), 1.0);
    }

    #[test]
    fn refresh_resets_to_ready() {
        let mut cooldown = Cooldown::from_secs(10.0);
        cooldown.trigger().unwrap();
        assert!(!cooldown.ready());
        cooldown.refresh();
        assert!(cooldown.ready());
        assert_eq!(cooldown.remaining(), 0.0);
    }

    #[test]
    fn multiple_trigger_cycles() {
        let mut cooldown = Cooldown::from_secs(1.0);

        // First cycle
        assert!(cooldown.trigger().is_ok());
        cooldown.tick(Duration::from_secs_f32(1.0));
        assert!(cooldown.ready());

        // Second cycle
        assert!(cooldown.trigger().is_ok());
        cooldown.tick(Duration::from_secs_f32(1.0));
        assert!(cooldown.ready());
    }

    // ==================== CooldownState Tests ====================

    #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
    enum TestAbility {
        Attack,
        Defend,
        Special,
    }

    #[test]
    fn cooldown_state_new_from_pairs() {
        let state = CooldownState::new([
            (TestAbility::Attack, Cooldown::from_secs(1.0)),
            (TestAbility::Defend, Cooldown::from_secs(2.0)),
        ]);

        assert!(state.get(&TestAbility::Attack).is_some());
        assert!(state.get(&TestAbility::Defend).is_some());
        assert!(state.get(&TestAbility::Special).is_none());
    }

    #[test]
    fn cooldown_state_default_is_empty() {
        let state: CooldownState<TestAbility> = CooldownState::default();
        assert!(state.get(&TestAbility::Attack).is_none());
    }

    #[test]
    fn cooldown_state_tracks_abilities_independently() {
        let mut state = CooldownState::new([
            (TestAbility::Attack, Cooldown::from_secs(1.0)),
            (TestAbility::Defend, Cooldown::from_secs(2.0)),
        ]);

        // Trigger only Attack
        state.trigger(&TestAbility::Attack).unwrap();

        // Attack should be on cooldown, Defend should be ready
        assert!(!state.ready(&TestAbility::Attack));
        assert!(state.ready(&TestAbility::Defend));
    }

    #[test]
    fn cooldown_state_trigger_unknown_ability_fails() {
        let mut state = CooldownState::new([(TestAbility::Attack, Cooldown::from_secs(1.0))]);

        // Special was never added
        assert_eq!(state.trigger(&TestAbility::Special), Err(CooldownNotReady));
    }

    #[test]
    fn cooldown_state_ready_returns_false_for_unknown_ability() {
        let state = CooldownState::new([(TestAbility::Attack, Cooldown::from_secs(1.0))]);

        // Special was never added, should return false (not panic)
        assert!(!state.ready(&TestAbility::Special));
    }

    #[test]
    fn tick_all_updates_all_cooldowns() {
        let mut state = CooldownState::new([
            (TestAbility::Attack, Cooldown::from_secs(1.0)),
            (TestAbility::Defend, Cooldown::from_secs(2.0)),
        ]);

        state.trigger(&TestAbility::Attack).unwrap();
        state.trigger(&TestAbility::Defend).unwrap();

        // Tick by 1 second
        state.tick_all(Duration::from_secs_f32(1.0));

        // Attack (1s cooldown) should be ready
        assert!(state.ready(&TestAbility::Attack));
        // Defend (2s cooldown) should still have 1s remaining
        assert!(!state.ready(&TestAbility::Defend));
        assert!((state.get(&TestAbility::Defend).unwrap().remaining() - 1.0).abs() < 0.001);
    }

    #[test]
    fn get_mut_allows_direct_modification() {
        let mut state = CooldownState::new([(TestAbility::Attack, Cooldown::from_secs(1.0))]);

        state.trigger(&TestAbility::Attack).unwrap();
        assert!(!state.ready(&TestAbility::Attack));

        // Use get_mut to refresh directly
        state.get_mut(&TestAbility::Attack).unwrap().refresh();
        assert!(state.ready(&TestAbility::Attack));
    }
}
