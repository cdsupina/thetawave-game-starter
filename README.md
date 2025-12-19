# Thetawave Game Starter

A complete starter template for creating Thetawave-style space shooter games using Bevy 0.17 and Rust. Features a modular crate architecture with extensive customization through extended data assets and the new `.mob` file asset system.

## Key Features
- **Complete Game Systems**: Character selection, physics, persistence, audio, particle effects
- **Extended Assets**: Override and extend game data (characters, mobs, projectiles) via TOML and `.mob` files
- **Modular Architecture**: 12 specialized crates for clean separation of concerns
- **Cross-Platform**: Supports desktop and WASM (WebGPU) deployment
- **Developer Friendly**: Debug menu with auto-generated spawn lists, world inspector, behavior tree visualization

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

### Mob Definitions (`.mob` Files)
Mobs are defined using individual `.mob` files (TOML format) instead of monolithic configuration files:
- **Base mobs**: `assets/mobs/` - 26 embedded mob definitions
- **Extended mobs**: `thetawave-test-game/assets/mobs/` - Add new mobs or override existing ones
- **Mob patches** (`.mobpatch`): Partial overrides that merge field-level into base mobs

Example `.mob` file:
```toml
name = "Xhitara Grunt"
colliders = [{ shape = { Rectangle = [12.0, 15.0] }, position = [0.0, 0.0], rotation = 0.0 }]
sprite_key = "xhitara_grunt_mob"

[behavior]
type = "Forever"
[[behavior.children]]
type = "Action"
name = "Movement"
behaviors = [{ action = "MoveDown" }, { action = "BrakeHorizontal" }]
```

### Character Data
Customize player characters in `thetawave-test-game/assets/data/character_attributes.toml`. Extended files merge with base assets using field-level overrides.

## Project Architecture

The workspace contains 12 specialized crates:

| Crate | Purpose |
|-------|---------|
| `thetawave-starter` | Main plugin, collision system, window management, UI orchestration |
| `thetawave-core` | Shared types: `Faction`, `HealthComponent`, `CollisionDamage`, app states |
| `thetawave-assets` | Asset loading with `bevy_asset_loader`, manages base and extended assets |
| `thetawave-player` | Input handling, ability system, character attributes |
| `thetawave-mobs` | Mob spawning, `.mob` file asset system, AI behaviors via `bevy_behave` |
| `thetawave-projectiles` | Projectile lifecycle, collision/damage, despawn ordering |
| `thetawave-particles` | Unified particle effects using `bevy-enoki` |
| `thetawave-physics` | Wraps `avian2d` physics, manages pause/resume |
| `thetawave-backgrounds` | Background and planet rendering |
| `thetawave-camera` | 2D/3D camera zoom via events |
| `thetawave-debug` | World inspector via `bevy-inspector-egui` (feature-gated) |
| `thetawave-test-game` | Example game binary |

The following dependency graph shows the modular architecture and relationships between crates:

![Dependency Graph](dependency-graph.png)

*The dependency graph is automatically generated during builds and shows the clean separation between game systems.*

## Credits

### Asset Attributions
- **Space Backgrounds**: [Seamless Space Backgrounds](https://screamingbrainstudios.itch.io/seamless-space-backgrounds) by Screaming Brain Studios
- **Music**: [Joel Schuman](https://joelhasa.site/)
- **Font**: [Dank Depths Pixel Font](https://hexany-ives.itch.io/dank-depths-pixel-font) by hexany-ives
- **Pixel Art**: [LordDeatHunter](https://github.com/LordDeatHunter)
