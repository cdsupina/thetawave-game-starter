# Summary
This game is a starter template for Thetawave space shooter games.
It contains all the required features of a Thetawave style game.
- [x] Main Menu
- [ ] Options Menu
  - [x] Display Resolutions
  - [x] Fullscreen
  - [ ] Graphics Settings
  - [ ] Arcade Mode
  - [ ] Control Rebinding
  - [ ] Volume Controls
- [x] 2D + 3D Backgrounds
- [x] Exit Game
- [x] Embedded Assets
- [ ] Loading Screens
- [ ] Character Selection
- [x] Pause Menu
- [ ] Save Games
- [ ] Background Music Player

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
