# Pond

A toy GPU renderer, written in Rust. The intent is to learn how a renderer/graphics pipeline works, leveraging modern APIs, abstractions and techniques.

## Status

This workspace is pretty barebones at the moment - WIP!

![Pond](/README/pond.png "WIP")

## Goals

- [ ] GPU-driven
- [ ] Bindless
- [ ] Physically-based (basic PBR implementation)
- [ ] Cross-platform (Vulkan + DX12)
- [ ] glTF support
- [ ] Render Graph-ready

## Non-Goals

- Content Pipeline
- Level Editor
- Scripting Support
- Gameplay Support
- ...anything else Game-Engine related; this is **only** a renderer!

## Crates

This workspace leverages existing crates enabling graphics programming in Rust, of particular note:

- [ash](https://crates.io/crates/ash): Vulkan bindings for Rust
- [spirv-std](https://crates.io/crates/spirv-std): Compile no-std Rust crates as Shaders (!!!!!!!)
- [glam](https://crates.io/crates/glam): SIMD-ready Linear algebra crate
- [gpu-allocator](https://crates.io/crates/gpu-allocator): CPU/GPU memory allocator for Vulkan/DX12
- [imgui](https://crates.io/crates/imgui): Dear ImGui bindings for Rust
- [winit](https://crates.io/crates/winit): Cross-platform windowing crate
- [gltf](https://crates.io/crates/gltf): glTF 2.0 loader
