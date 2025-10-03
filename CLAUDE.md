# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a Bevy-based space shooter game template built with Rust. The project uses a modular crate architecture where game systems are separated into specialized crates with clear dependencies. The game supports both native (desktop) and WASM (web) deployment with an extensible data system that allows overriding game configuration via TOML files.

## Build & Run Commands

### Development

**Native builds:**
```bash
# Run from workspace root (base data only - extended data won't load)
cargo run --bin thetawave-test-game

# Run from thetawave-test-game directory (recommended - enables extended data)
cd thetawave-test-game
cargo run
```

**WASM builds:**
```bash
cd thetawave-test-game
trunk serve  # Development server with hot reload at http://localhost:8080
```

### Release Builds

**Desktop:**
```bash
cd thetawave-test-game
cargo build --release --no-default-features
```

**WASM:**
```bash
cd thetawave-test-game
trunk build  # Output in dist/ directory
```

### Testing & Verification

```bash
cargo check          # Fast compile check
cargo build          # Full build
cargo clean          # Clean build artifacts
```

## Modular Crate Architecture

The project uses a workspace with 15 specialized crates:

- **thetawave-test-game**: Main binary entry point, minimal main.rs that configures and runs the game
- **thetawave-starter**: Core plugin orchestration, registers all game plugins and sets up the Bevy app with window configuration, asset sources, and plugin ordering
- **thetawave-core**: Shared types, state management (AppState, GameState, MainMenuState, PauseMenuState, DebugState), faction system, health, collision damage, and the critical `load_with_extended` utility for TOML merging
- **thetawave-assets**: Asset loading and management for sprites, audio, UI, backgrounds; handles both embedded base assets and extended assets via "extended://" asset source
- **thetawave-player**: Player entities, character selection, input handling (keyboard/gamepad), abilities system using leafwing_abilities, and player stats
- **thetawave-mobs**: Enemy entities, AI behavior trees (bevy_behave), mob spawning, and death events
- **thetawave-projectiles**: Projectile spawning, collision shapes, spread patterns, and projectile types
- **thetawave-physics**: Physics integration with Avian2D, collision layers, and collider management
- **thetawave-particles**: Particle effects system using bevy_enoki, blood effects, and spawner-based particles
- **thetawave-backgrounds**: Scrolling background system with parallax effects
- **thetawave-camera**: Camera following and viewport management
- **thetawave-debug**: Debug UI with bevy-inspector-egui (debug feature only)

**Key architectural patterns:**
- Each crate exports a plugin (e.g., `ThetawaveMobsPlugin`) that gets added in thetawave-starter
- State-based system scheduling: systems run conditionally based on AppState, GameState, etc.
- Event-driven architecture: SpawnMobEvent, PlayerDeathEvent, ExecutePlayerAbilityEvent, etc.
- Data-driven design: TOML files define character stats, mob behaviors, projectile attributes

## Extended Data System

The game uses a two-tier asset system:

1. **Base data**: Embedded in binary via `include_bytes!()` macros
2. **Extended data**: Optional TOML files in `thetawave-test-game/assets/data/` that merge with base data

**Field-level merging:**
- Extended TOML files are merged with base data at the field level
- If an entry exists in base, only the specified fields in extended override
- If an entry doesn't exist in base, the entire entry is added
- Implemented in `thetawave-core/src/data_loader.rs` via `load_with_extended<T>()`

**Extended data files:**
- `character_attributes.toml` - Player character stats, abilities, sprites
- `mob_attributes.toml` - Enemy health, projectiles, sprites
- `mob_behaviors.toml` - AI behavior tree definitions
- `projectile_attributes.toml` - Projectile physics and collision shapes

**Platform differences:**
- Native: Uses filesystem access to load `assets/data/*.toml`
- WASM: Uses HTTP requests to fetch files from server
- Both work identically from the code perspective

**Working directory context:**
- Extended data files only load when running from `thetawave-test-game/` directory
- Running from workspace root won't load extended data (filesystem path won't resolve)
- WASM builds always work because assets are served via HTTP

## Asset System Architecture

**Asset sources:**
- Default source: Embedded assets via bevy_embedded_assets (base game data)
- "extended://" source: Registered in thetawave-starter for native (FileAssetReader) and WASM (HttpWasmAssetReader)

**Media assets:**
- `ui.assets.ron` - UI textures, fonts
- `game.assets.ron` - Sprites, animations (Aseprite format via bevy_aseprite_ultra)
- `music.assets.ron` - Audio files (mp3/wav via bevy_kira_audio)
- `background.assets.ron` - Background textures

## State Management

The game uses Bevy's state system with multiple orthogonal states:

- **AppState**: MainMenu | Game
- **GameState**: Playing | Paused (only relevant when AppState::Game)
- **MainMenuState**: Title | CharacterSelection (only relevant when AppState::MainMenu)
- **PauseMenuState**: Various pause menu screens
- **DebugState**: Normal | Debug (toggled via Backquote key by default)

Systems use `.run_if()` with state conditions to control execution.

## Key Dependencies

- **Bevy 0.16.1**: Game engine (nightly Rust required for edition 2024)
- **avian2d**: Physics engine (formerly Rapier)
- **bevy_enoki**: Particle system
- **bevy_aseprite_ultra**: Aseprite sprite format support
- **bevy_behave**: Behavior trees for AI
- **leafwing_abilities**: Cooldown-based ability system
- **leafwing-input-manager**: Input handling abstraction
- **bevy-persistent**: Save data management

## Common Patterns

**Adding a new mob type:**
1. Add entry to `mob_attributes.toml` (base or extended) with health, projectiles, sprite
2. Add behavior to `mob_behaviors.toml` if custom AI needed
3. Spawn via `SpawnMobEvent` - system in thetawave-mobs handles instantiation

**Adding a new player ability:**
1. Add ability to PlayerAbility enum in thetawave-player
2. Register system via `extended_abilities` HashMap in ThetawavePlayerPlugin
3. Add to `extended_duration_abilities` if it's a duration-based ability
4. Update character TOML with new ability reference

**Particle effects:**
- Use bevy_enoki for particles
- Blood effects trigger on MobDeathEvent
- Particle colors use `Color::srgba()` - values can exceed 1.0 for bloom effect

## Addtional Notes - IMPORTANT
- After editing files make sure to use the Rust clippy linter and address any warnings
