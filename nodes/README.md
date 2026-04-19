# Nodes

Rust game workspace scaffold.

## Crates

- `game_core`: gameplay rules and state. Keep this free of windowing, GPU, filesystem paths, and platform APIs.
- `game_render`: rendering layer. `wgpu` setup, pipelines, shaders, and frame drawing should live here.
- `game_ui`: UI layer. `egui` HUD, debug panels, editor tools, and menus should live here.
- `game_platform`: platform services. Assets, storage, logging, permissions, audio hooks, and platform-specific paths should be abstracted here.
- `game_app`: executable shell. `winit` event loop, app lifecycle, input collection, and crate wiring should live here.

## Platform Wrappers

- `platforms/android`: Android Gradle, manifest, Activity, signing, and NDK packaging.
- `platforms/ios`: Xcode project, app delegate, signing, and IPA packaging.
- `platforms/desktop`: Windows, Linux, and macOS packaging notes/scripts.

