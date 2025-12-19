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
| `thetawave-starter` | Main plugin, collision system, window management, UI orchestration, audio |
| `thetawave-core` | Shared types: `Faction`, `HealthComponent`, `CollisionDamage`, app states, TOML merging, logging |
| `thetawave-assets` | Asset loading with `bevy_asset_loader`, manages base (embedded) and extended (filesystem) assets |
| `thetawave-player` | Input handling (`leafwing-input-manager`), ability registry pattern, character attributes |
| `thetawave-mobs` | `.mob` file asset system, `MobRegistry`, AI behaviors via `bevy_behave`, projectile spawners |
| `thetawave-projectiles` | Projectile lifecycle, collision/damage, despawn ordering via `ProjectileSystemSet` |
| `thetawave-particles` | Unified particle effects using `bevy-enoki`, event-driven spawning |
| `thetawave-physics` | Wraps `avian2d` physics, manages pause/resume based on game state |
| `thetawave-backgrounds` | Background and planet rendering using Bevy 3D |
| `thetawave-camera` | 2D/3D camera zoom via events |
| `thetawave-debug` | World inspector via `bevy-inspector-egui`, auto-generated spawn menu (feature-gated) |
| `thetawave-test-game` | Example game binary |

### Key Patterns

**Event-Driven Communication**: Plugins communicate via Bevy messages (e.g., `SpawnMobEvent`, `SpawnProjectileEvent`, `PlayerDeathEvent`, `MobDeathEvent`, `SpawnerParticleEffectSpawnedEvent`). Register with `app.add_message::<T>()`.

**Dual Asset System**: Base assets are embedded in the binary via `bevy_embedded_assets`. Extended assets load from `assets/` directory at runtime using the `"extended://"` asset source, allowing customization without recompilation.

**State Hierarchy**:
- `AppState`: MainMenuLoading → MainMenu → GameLoading → Game
- `MainMenuState`: Title, Options, InputRebinding, CharacterSelection
- `GameState`: Playing, Paused, End
- `PauseMenuState`: substates for pause menu navigation
- `DebugState`: None, Debug (feature-gated)
- Systems use `run_if(in_state(AppState::Game).and(in_state(GameState::Playing)))`

**Marker Components for Queries**:
- `PlayerStats` - identifies player entities
- `MobMarker` - identifies mob entities (contains `mob_ref` string)
- `ProjectileType` - identifies projectiles
- `JointsComponent` - tracks jointed mob relationships

**Ability System Extension**: Custom abilities injected via `ThetawaveStarterPlugin::extended_abilities` HashMap mapping ability names to `SystemId<In<Entity>>`. Duration abilities tracked in `extended_duration_abilities` HashSet.

## .mob File Asset System

Mobs are defined using individual `.mob` files (TOML format) loaded as Bevy assets:

### File Types
- **`.mob` files**: Complete mob definitions
- **`.mobpatch` files**: Partial overrides that merge field-level into base mobs

### Asset Loading Pipeline
1. Base `.mob` files registered in `assets/mobs.assets.ron`
2. Extended `.mob` files in `thetawave-test-game/mobs.assets.ron`
3. Extended `.mobpatch` files override base mobs at field level
4. `MobRegistry` built at `OnEnter(AppState::Game)`:
   - Loads raw TOML values from base mobs
   - Adds extended complete mobs
   - Merges `.mobpatch` files into values
   - Deserializes to `MobAsset` structs
   - Pre-builds behavior trees for each mob

### MobAsset Structure (key fields)
```rust
pub struct MobAsset {
    pub name: String,
    pub spawnable: bool,                    // appears in debug spawn menu
    pub colliders: Vec<ThetawaveCollider>,
    pub z_level: f32,
    pub max_linear_speed: Vec2,
    pub linear_acceleration: Vec2,
    pub projectile_spawners: Option<ProjectileSpawnerComponent>,
    pub jointed_mobs: Option<Vec<JointedMobRef>>,  // for complex mobs
    pub behavior: BehaviorNodeData,
    pub sprite_key: Option<String>,
    pub decorations: Vec<[String; 2]>,
}
```

### Example .mob File
```toml
name = "Xhitara Grunt"
colliders = [{ shape = { Rectangle = [12.0, 15.0] }, position = [0.0, 0.0], rotation = 0.0 }]
sprite_key = "xhitara_grunt_mob"
decorations = [["xhitara_grunt_thrusters", [0.0, 10.0]]]

[behavior]
type = "Forever"
[[behavior.children]]
type = "Action"
name = "Movement"
behaviors = [{ action = "MoveDown" }, { action = "BrakeHorizontal" }]
```

### Example .mobpatch File
```toml
name = "Super Fast Spitter"
projectile_speed = 300.0
[projectile_spawners.spawners.south]
timer = 0.25
```

## Character Data Configuration

Character attributes defined in TOML (`thetawave-player/data/character_attributes.toml`):
```toml
[characters.captain_character]
acceleration = 2.0
deceleration = 0.972
max_speed = 100.0
collider_dimensions = [6.0, 12.0]
health = 150
projectile_speed = 200.0
projectile_count = 1
projectile_spread = { Arc = {...} }

[characters.captain_character.cooldowns]
BasicAttack = 0.5
SecondaryAttack = 1.5

[characters.captain_character.abilities]
BasicAttack = "fire_blast"
SecondaryAttack = "mega_blast"
```

Extended data merges with base via `load_with_extended()` from `thetawave-core`.

## Feature Flags

| Feature | Purpose | Default |
|---------|---------|---------|
| `debug` | Enables bevy-inspector-egui world inspector and debug menu | ON in test-game |
| `dynamic_linking` | Faster iteration via dynamic Bevy linking | ON in test-game |
| `physics_debug` | Physics diagnostic UI (avian2d) | OFF |

## Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| bevy | 0.17.3 | Core game engine |
| bevy_asset_loader | 0.24.0 | Asset loading with states |
| iyes_progress | 0.15.0 | Loading progress tracking |
| avian2d | 0.4.1 | 2D physics engine |
| bevy_behave | 0.4.0 | Behavior tree AI system |
| bevy-enoki | 0.5 | Particle effects |
| leafwing-input-manager | 0.19.0 | Input handling |
| bevy_aseprite_ultra | 0.7.0 | Aseprite sprite animation |
| bevy_kira_audio | 0.24.0 | Audio system |
| bevy-inspector-egui | 0.35.0 | Debug world inspector |
| toml | 0.9.8 | TOML parsing for data files |

## Rendering Notes

- Color::srgba values may exceed 1.0 for bloom effect
- Use `with_bloom(color, bloom_factor)` utility for faction colors
- Nearest neighbor filtering for crisp pixel art: `ImagePlugin::default_nearest()`
- Faction colors: `ALLY_BASE_COLOR` (yellow), `ENEMY_BASE_COLOR` (red), `XHITARA_BLOOD_COLOR` (cyan)
