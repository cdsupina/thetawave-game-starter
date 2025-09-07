# Thetawave Game Starter

A complete starter template for creating Thetawave-style space shooter games using Bevy and Rust. Features a modular crate architecture with extensive customization through extended data assets.

## Key Features
- **Complete Game Systems**: Character selection, physics, multiplayer, persistence, audio
- **Extended Assets**: Override and extend game data (characters, mobs, projectiles, behaviors) via TOML files
- **Modular Architecture**: 16 specialized crates for clean separation of concerns
- **Cross-Platform**: Supports desktop and WASM deployment
- **Developer Friendly**: Extended assets and data, built in debug menu.

## Quick Start

### Prerequisites
- Rust nightly toolchain
- Game assets (contact cdsupina@gmail.com for media files)
- For WASM builds: [Trunk](https://trunkrs.dev/) (`cargo install trunk`)

### Setup
1. Clone repository and place media files in `assets/media/`
2. Install Cranelift for faster compilation:
   ```bash
   rustup component add rustc-codegen-cranelift-preview --toolchain nightly
   ```

### Development

#### Native Builds
```bash
# Run with extended data files (recommended for development)
# NOTE: Must run from thetawave-test-game directory for extended data files to work
cd thetawave-test-game
cargo run

# Or run from workspace root (uses base data only - extended data files won't load)
cargo run --bin thetawave-test-game
```

#### WASM Development
```bash
# Serve WASM build locally with hot reload
cd thetawave-test-game
trunk serve

# The game will be available at http://localhost:8080
# Extended data files are automatically accessible via HTTP in WASM builds
```

### Release Builds

#### Desktop Release
```bash
# Build from thetawave-test-game directory for extended data file support
cd thetawave-test-game
cargo build --release --no-default-features
```

#### WASM Release
```bash
# Build optimized WASM bundle with Trunk
cd thetawave-test-game
trunk build

# Output will be in dist/ directory
# Deploy dist/ contents to web server for production
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

Extended files merge with base assets using field-level overrides.

## Project Architecture

The following dependency graph shows the modular architecture and relationships between crates:

![Dependency Graph](dependency-graph.png)

*The dependency graph is automatically generated during builds and shows the clean separation between game systems.*
