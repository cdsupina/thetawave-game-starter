# Summary
This game is a starter template for Thetawave space shooter games.
It contains all the required features of a Thetawave style game.
- [x] Main Menu
- [ ] Options Menu
  - [x] Display Resolutions
  - [x] Fullscreen
  - [ ] Graphics Settings
  - [ ] Control Rebinding
  - [x] Volume Controls
- [x] 2D + 3D Backgrounds
- [x] Exit Game
- [x] Embedded Assets
- [x] Loading Screens
- [ ] Character Selection
- [x] Pause Menu
- [ ] Persistence
  - [x] Save and load settings
  - [ ] Save overall progress
- [x] Audio Playing
  - [x] Background Music Channel
  - [x] Effects Channel
  - [x] Ui Channel
- [x] 2D Physics
- [ ] Local Multiplayer
- [ ] Win and Loss Screens
- [ ] Logo Splash Screens
- [ ] WASM Deployment

# Build and Run
## Download Assets
Email cdsupina@gmail.com for access to pCloud game assets.
Copy the media directory into the assets directory.

## Install Cranelift
```bash
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
```
## Build
```bash
cargo build
```
## Run
```bash
cargo run
```

## Release
Do not use default features for release build so that dynamic linking is disabled.
```bash
cargo build --release --no-default-features
```
