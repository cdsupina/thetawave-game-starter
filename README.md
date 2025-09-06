# Thetawave Game Starter

A complete starter template for creating Thetawave-style space shooter games using Bevy and Rust. Features a modular crate architecture with extensive customization through extended data assets.

## Key Features
- **Complete Game Systems**: Character selection, physics, multiplayer, persistence, audio
- **Extended Assets**: Override and extend game data (characters, mobs, projectiles, behaviors) via TOML files
- **Modular Architecture**: 16 specialized crates for clean separation of concerns
- **Cross-Platform**: Supports desktop and WASM deployment
- **Developer Friendly**: Hot-reloadable assets and comprehensive debugging tools

## Quick Start

### Prerequisites
- Rust nightly toolchain
- Game assets (contact cdsupina@gmail.com for media files)

### Setup
1. Clone repository and place media files in `assets/media/`
2. Install Cranelift for faster compilation:
   ```bash
   rustup component add rustc-codegen-cranelift-preview --toolchain nightly
   ```

### Development
```bash
# Run with extended data assets (recommended for development)
cd thetawave-test-game
cargo run

# Or run from workspace root (uses base assets only)
cargo run --bin thetawave-test-game
```

### Release Builds
```bash
# Desktop release
cargo build --release --no-default-features

# WASM build
cargo build --release --no-default-features --target wasm32-unknown-unknown
```

## Extended Assets System

### Media Assets
Override or extend media assets by creating files in `thetawave-test-game/assets/`:
- `ui.assets.ron` - UI textures, fonts, and interface assets
- `game.assets.ron` - Sprites, animations, and game visuals  
- `music.assets.ron` - Audio files and sound effects
- `background.assets.ron` - Background textures and environment art
- `media/` - Place actual asset files (images, audio, etc.)

### Data Assets  
Customize game behavior with TOML configuration files in `thetawave-test-game/assets/data/`:
- `character_attributes.toml` - Player character stats and abilities
- `mob_attributes.toml` - Enemy attributes, health, and projectiles
- `mob_behaviors.toml` - AI behavior trees and movement patterns
- `projectile_attributes.toml` - Projectile physics and collision shapes

Extended files merge with base assets using field-level overrides, preserving defaults while allowing selective customization.
