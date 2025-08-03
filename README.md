# apogeetrak

A 3D satellite tracking simulation built using the [Bevy](https://bevy.org/) game engine and [SGP4](https://github.com/neuromorphicsystems/sgp4) crate for orbital mechanics.
This was a fun first project for me in familiarizing myself with the Rust workflow and exploring various PBR techniques.

## Install

```bash
git clone https://github.com/yourusername/apogeetrak.git
cd apogeetrak
cargo run --release
```

The executable will be found under `target/release/`

## Controls

- **Right-click + drag** - Rotate camera
- **Mouse wheel** - Zoom
- **Time buttons** - Control simulation speed

## Notes

You can tweak simulation settings in `src/config.rs` for things like terrain height, atmospheric parameters, rotation speed, etc.
You can also try out more TLE datasets from [NORAD](https://celestrak.org/NORAD/elements/), though you will need to replace the file reference in `systems/satellites/mod.rs`.

Do check out the WGSL shader code

---

*Built with Bevy 0.16 • SGP4 orbital mechanics • Custom WGSL shaders*