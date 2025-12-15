# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Development (from workspace root - uses dynamic linking for fast iteration)
cargo run --bin thetawave-test-game

# Development with extended data files (must run from thetawave-test-game directory)
cd thetawave-test-game && cargo run

# Check compilation
cargo check

# Run tests
cargo test

# Release build (from thetawave-test-game for extended data support)
cd thetawave-test-game && cargo build --release --no-default-features

# WASM build (requires trunk: cargo install trunk)
cd thetawave-test-game && trunk serve   # Dev with hot reload
cd thetawave-test-game && trunk build   # Production build to dist/
```

## Architecture Overview

This is a Bevy 0.17 game engine project organized as a Cargo workspace with 12 crates. The main plugin (`ThetawaveStarterPlugin` in `thetawave-starter`) orchestrates all subsystems.

### Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| `thetawave-starter` | Main plugin, collision system, window management, UI orchestration |
| `thetawave-core` | Shared types: `Faction`, `HealthComponent`, `CollisionDamage`, app states, data loading utilities |
| `thetawave-assets` | Asset loading with `bevy_asset_loader`, manages base (embedded) and extended (filesystem) assets |
| `thetawave-player` | Input handling (`leafwing-input-manager`), ability system with registry pattern, character attributes |
| `thetawave-mobs` | Mob spawning, AI behaviors via `bevy_behave`, projectile spawners |
| `thetawave-projectiles` | Projectile lifecycle, collision/damage, despawn ordering via `ProjectileSystemSet` |
| `thetawave-particles` | Unified particle effects using `bevy-enoki`, event-driven spawning |
| `thetawave-physics` | Wraps `avian2d` physics, manages pause/resume based on game state |
| `thetawave-backgrounds` | Background and planet rendering |
| `thetawave-camera` | 2D/3D camera zoom via events |
| `thetawave-debug` | World inspector via `bevy-inspector-egui` (feature-gated) |
| `thetawave-test-game` | Example game binary |

### Key Patterns

**Event-Driven Communication**: Plugins communicate via Bevy messages (e.g., `SpawnProjectileEvent`, `PlayerDeathEvent`, `SpawnBloodEffectEvent`). Register with `app.add_message::<T>()`.

**Dual Asset System**: Base assets are embedded in the binary. Extended assets load from `assets/` directory at runtime, allowing customization without recompilation.

**Data-Driven Configuration**: Game behavior defined in TOML files (`assets/data/`):
- `character_attributes.toml` - Player stats and abilities
- `mob_attributes.toml` - Enemy attributes
- `mob_behaviors.toml` - AI behavior trees
- `projectile_attributes.toml` - Projectile properties

Extended data merges with base data via `load_with_extended()` from `thetawave-core`.

**State Hierarchy**:
- `AppState`: MainMenuLoading → MainMenu → GameLoading → Game
- `GameState`: Playing, Paused, End
- Systems use `run_if(in_state(AppState::Game).and(in_state(GameState::Playing)))`

**Marker Components for Queries**:
- `PlayerStats` - identifies player entities
- `MobMarker` - identifies mob entities
- `ProjectileType` - identifies projectiles

**Ability System Extension**: Custom abilities injected via `ThetawaveStarterPlugin::extended_abilities` HashMap mapping ability names to `SystemId<In<Entity>>`.

## Rendering Notes

Color::srgba values may exceed 1.0 to bloom the color.
