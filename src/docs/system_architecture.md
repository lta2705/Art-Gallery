# System Architecture & Technical Flow Guide: 3D Art Gallery Project

**Author:** Senior Business Analyst
**Project:** 3D Art Gallery (Rust + OpenGL)
**Date:** March 31, 2026

---

## 1. Executive Summary
This project is a high-performance 3D visualization of an Art Gallery developed using the Rust programming language. It leverages the `glow` (GL on Whatever) library for hardware-accelerated rendering and `winit/glutin` for cross-platform windowing. The core business value is to provide an immersive 3D environment with dynamic lighting, interactive navigation, and multi-mode camera systems.

---

## 2. File & Module Responsibilities

| File Path | Component | Responsibility | Key Logic |
| :--- | :--- | :--- | :--- |
| `src/main.rs` | **Orchestrator** | Entry point, event loop, and high-level render integration. | Window creation, OpenGL context sync, Frame loop. |
| `src/geometry.rs` | **Module A** | Geometry definition and GPU buffer management. | Vertex/Index layouts, Mesh (VAO/VBO), Icosphere generation. |
| `src/camera.rs` | **Module B** | Spatial navigation and Coordinate Space transformations. | View/Projection matrices, WASD Physics, POV/CCTV modes. |
| `src/lighting.rs` | **Module C** | Illumination logic and point light management. | Attenuation formula, Light data serialization to Shaders. |
| `src/shader.rs` | **Utility** | Shader compilation and Uniform interface. | GLSL Program linking, Uniform set methods (mat4, vec3). |
| `src/texture.rs` | **Utility** | Asset loading and GPU texture binding. | Image decoding via `image` crate, Mipmap generation. |
| `shaders/*.glsl` | **GPU Kernel** | Low-level Phong reflection implementation. | Ambient/Diffuse/Specular calculations per fragment. |

---

## 3. Operational Data Flow

### A. Initialization Sequence (Setup Flow)
1.  **Context Boot**: `main.rs` starts the `winit` event loop and creates a `GLContext` via `glutin`.
2.  **Resource Loading**:
    *   **Shaders**: `shader.rs` compiles `.vert` and `.frag` files.
    *   **Textures**: `texture.rs` loads `art1.jpg`, `art2.jpg`, etc., and generates IDs.
    *   **Geometry**: `geometry.rs` generates the Hallway and Sphere data, uploading them to VBOs/EBOs.
3.  **State Initialization**: `Camera` and `PointLight` structs are instantiated with default world coordinates.

### B. The Engine Loop (Frame Flow)
Each frame follow these four phases:
1.  **Input Event Phase**: `main.rs` captures keyboard states (WASD) and mouse movement offsets.
2.  **Update Phase**: 
    *   `Camera` updates `head_pos` based on delta time and input.
    *   LookAt vectors are recalculated from current `Yaw` and `Pitch`.
3.  **Rendering Phase**:
    *   **Clear**: GPU buffers are cleared (Depth + Color).
    *   **Uniform Upload**: Global matrices (View, Projection) and Light arrays are sent to the GPU.
    *   **Draw Calls**: 
        *   Draw Hallway walls (switching textures for specific walls).
        *   Draw "The Head" (Sphere) with high specular highlight.
        *   Draw Light Bulbs using emissive shaders.
4.  **Buffer Swap**: The back buffer is swapped to the display.

---

## 4. In-Depth Technical Guide

### 4.1 Coordinate Space Transitions
Understanding how a pixel reaches the screen is vital for this project:
-   **Model Space**: Coordinates defined in `geometry.rs` (e.g., center of the sphere at 0,0,0).
-   **World Space**: Application of the `u_model` matrix in `room.vert`. The Head is translated to `camera.head_pos`.
-   **View Space**: Transformation into the camera's perspective using `camera.view_matrix()`. This is where "CCTV" vs "POV" logic diverges.
-   **Projection Space**: Conversion into 2D clip coordinates using the Perspective matrix (FOV, Aspect Ratio).

### 4.2 The Phong Reflection Model (Module C)
Our shaders implement the Phong model in `shaders/room.frag` and `shaders/sphere.frag`:
-   **Ambient**: A static fraction of light (e.g., 0.08) ensuring shadowed areas aren't pitch black.
-   **Diffuse**: Calculated using the Dot Product of the Surface Normal and Light Direction. Larger angles = dimmer light.
-   **Specular**: The most complex part. It depends on the `View Direction`, creating a highlight that moves as the user rotates the mouse.
-   **Attenuation**: Essential for the "Art Gallery" vibe. Distance squared calculation $1/(c + l \cdot d + q \cdot d^2)$ ensures the hallway ends in atmospheric darkness.

### 4.3 Multi-Camera Management
The system achieves "Dual-Camera Toggle" (Module B) via a simple Enum state:
-   **POV**: The `ViewMatrix` eye position is parented to the movement head.
-   **CCTV**: The `ViewMatrix` is overridden by static vectors, providing security-footage style fixed orientation.

---

## 5. Potential Business Improvements (Backlog)
-   [ ] **Optimization**: Frustum Culling to avoid rendering walls behind the POV.
-   [ ] **Feature**: Dynamic Art Loading - Fetching image URLs via HTTP to update gallery textures.
-   [ ] **UX**: Collision Detection - Logic to prevent "The Head" from passing through walls.

---
*End of Documentation*
