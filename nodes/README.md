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

## Current Prototype

The current prototype starts with JSON-driven node templates. Run:

```text
cargo run -p nodes_app
```

The app loads:

```text
assets/node_templates/ore_source.json
```

and converts it into a `NodeRenderPlan` with node size, port positions, colors, and property rows.

`nodes_app` opens a native `winit` window and renders the node editor through `egui-wgpu`.
The editor currently starts with two node instances, has a `+ Node` button, supports dragging nodes by their header, and creates links by dragging from an output port to an input port.
Links are straight polylines. Double-click a link segment to insert a waypoint, drag the waypoint to route the link, right-click a waypoint to remove it, and right-click a link segment to cut the link.

