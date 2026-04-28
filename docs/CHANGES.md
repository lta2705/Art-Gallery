# 3D Art Gallery - Implementation Changes Log

## Project: 3D Art Gallery (Rust + OpenGL)

**Date:** April 26, 2026  
**Status:** Updated - Room Dimensions & Lighting Control

---

## 1. Room Dimensions Update

### Previous Dimensions
- Height: 3.0m
- Shape: L-shaped (4m × 20m main + 8m × 4m branch)
- Coordinates: p1[-2,-10] → p2[2,-10] → p3[2,2] → p4[10,2] → p5[10,6] → p6[-2,6]

### New Dimensions (per PDF specification / Procedural Code)
- **Height: 3.2m**
- **Shape:** L-shaped (9.5m × 3m main + 3.5m × 3.5m branch)
- **Coordinates (Procedural Space):**
  ```text
       p6(0,-6.5) --------- p5(3.5,-6.5)
       |                   |
       |   Branch (3.5×3.5)|
       |                    |
       |                    p4(3.5,-3) ----- p3(9.5,-3)
       |                                     
       |        Main Hallway (9.5×3.0)       
       |                                     
  p1(0,0) -------------------------------- p2(9.5,0)
  ```

| Point | X | Z |
|-------|---|---|
| p1 | 0.0 | 0.0 |
| p2 | 9.5 | 0.0 |
| p3 | 9.5 | -3.0 |
| p4 | 3.5 | -3.0 |
| p5 | 3.5 | -6.5 |
| p6 | 0.0 | -6.5 |

### Raw STL Dimensions (Loaded from File)
- **Shape:** Same L-shape but rotated 90 degrees and translated.
- **Bounds:** X ∈ [0, 6.5], Z ∈ [-9.5, 0]
- **Coordinates (Original STL Space):**
  ```text
                               p3(3.5,-9.5) -------- p2(6.5,-9.5)
                                    |                    |
                                    |                    |
                                    |                    |
                                    |    Main Hallway    |
  p5(0,-3.5) --------- p4(3.5,-3.5) |     (9.5×3.0)      |
       |                    |                            |
       |  Branch (3.5×3.5)  |                            |
       |                    |                            |
  p6(0,0) ------------------------------------------ p1(6.5,0)
  ```

| Point | X (STL) | Z (STL) |
|-------|---------|---------|
| p1 | 6.5 | 0.0 |
| p2 | 6.5 | -9.5 |
| p3 | 3.5 | -9.5 |
| p4 | 3.5 | -3.5 |
| p5 | 0.0 | -3.5 |
| p6 | 0.0 | 0.0 |

---

## 2. Files Modified

### `src/geometry.rs` (Line ~103-116)
```rust
// BEFORE:
const H: f32 = 3.0;
let p1 = [-2.0, -10.0];
let p2 = [2.0, -10.0];
// ... old L-shape

// AFTER:
const H: f32 = 3.2;  // Ceiling height: 3.2m
let p1 = [0.0, 0.0];
let p2 = [9.5, 0.0];
let p3 = [9.5, -3.0];
let p4 = [3.5, -3.0];
let p5 = [3.5, -6.5];
let p6 = [0.0, -6.5];
```

### `src/lighting.rs` (Line ~54-60)
```rust
// BEFORE: 5 point lights along hallway

// AFTER: 2 accent lights at corners
vec![
    PointLight::bright_warm(Vec3::new(3.5, 2.9, -3.0)),  // Inner corner
    PointLight::bright_warm(Vec3::new(9.0, 2.9, -1.5)),   // End of main
]
```

### `src/camera.rs` (Line ~37-50)
```rust
// BEFORE:
head_pos: Vec3::new(3.25, 0.5, -2.0),
cctv_pos: Vec3::new(3.25, 3.0, -0.5),
cctv_target: Vec3::new(3.25, 0.0, -5.0),

// AFTER:
head_pos: Vec3::new(4.0, 0.5, -1.5),      // Center of main hallway
cctv_pos: Vec3::new(4.75, 3.0, -1.5),    // Ceiling center
cctv_target: Vec3::new(4.75, 0.0, -3.25),
```

### `src/main.rs` - Multiple Changes

| Change | Location | Details |
|--------|----------|---------|
| Ceiling light position | Line ~407 | `(4.75, 3.1, -1.5)` - center of main hallway |
| Collision bounds | Line ~340-380 | Updated for new 9.5m × 6.5m L-shape |
| Painting positions | Line ~185-202 | 3 paintings on main walls and branch |
| Furniture position | Line ~466 | `(1.5, 0, -5.5)` - branch corner |
| Lighting controls | Line ~289-292 | L, [, ], P keys added |
| Lighting state | Line ~21-32 | New struct for runtime control |

---

## 3. Lighting Control System

### New Keyboard Controls

| Key | Action |
|-----|--------|
| `L` | Toggle spotlight on/off |
| `[` | Decrease spotlight intensity (×0.9) |
| `]` | Increase spotlight intensity (×1.1, max 5.0) |
| `P` | Toggle point lights on/off |
| `C` | Toggle POV ↔ CCTV (existing) |
| `WASD` | Move head (existing) |
| `Mouse` | Look around (existing) |

### Lighting Features
- **Spotlight:** Ceiling-mounted, points straight down, adjustable intensity
- **Point Lights:** 2 accent lights at corners (toggleable)
- **Shadow Mapping:** PCF 3×3 filtering, 2048×2048 depth map
- **Anti-aliasing:** 4x MSAA enabled

---

## 4. Object Arrangement

### Painting Positions (Updated)

| Art | Position | Wall |
|-----|----------|------|
| Art 1 | (0.1, 1.5, -1.5) | Left wall of main hallway |
| Art 2 | (9.4, 1.5, -1.5) | Right wall of main hallway |
| Art 3 | (1.75, 1.5, -6.4) | Back wall of branch hallway |

### Furniture Position
- **Location:** (1.5, 0, -5.5) - Branch hallway corner
- **Model:** furniture.stl

### Ceiling Light Bulb
- **Position:** (4.75, 3.1, -1.5) - Center of main hallway ceiling
- **Visual:** Emissive sphere mesh (fallback) or 3d-model.obj

---

## 5. Collision System Update

### New L-Shape Bounds (9.5m × 6.5m)
```
Main Hallway: x [0, 9.5], z [-3, 0]
Branch:       x [0, 3.5], z [-6.5, -3]

With head radius 0.4m + 0.1m shell:
- Main:  x ∈ [0.5, 9.0],  z ∈ [-2.6, -0.5]
- Branch: x ∈ [0.5, 3.0], z ∈ [-6.1, -3.5]
```

---

## 6. General Requirements Checklist (Updated)

| Requirement | Status | Notes |
|-------------|--------|-------|
| At least 1 Image on wall | ✅ DONE | 3 paintings with art textures |
| Room imported from STL | ✅ DONE | "The art gallery.stl" support |
| Ball pretending as human head | ✅ DONE | UV Sphere |
| Ball moving via keyboard | ✅ DONE | WASD/Arrow keys |
| Change looking direction by mouse | ✅ DONE | Mouse look |
| Switch POV ↔ CCTV with mouse | ✅ DONE | Press 'C' key |
| At least 1 furniture (STL) | ✅ DONE | furniture.stl |
| Shadow mapping | ✅ DONE | PCF 3×3, 2048×2048 |
| Anti-aliasing | ✅ DONE | 4x MSAA |
| Lighting system | ✅ DONE | Phong/Blinn-Phong + Spotlight |
| Collision handling | ✅ DONE | AABB for L-shape room |
| Lighting control | ✅ NEW | L, [, ], P keys |
| Room dimensions | ✅ UPDATED | 9.5m × 6.5m × 3.2m |

---

## 7. Summary

**Changes Made:**
1. ✅ Room height: 3.0m → 3.2m
2. ✅ Room coordinates: New L-shape (9.5m × 3m main + 3.5m × 3.5m branch)
3. ✅ Ceiling light: Positioned at center of main hallway
4. ✅ CCTV camera: Updated to ceiling center position
5. ✅ Collision bounds: Updated for new room shape
6. ✅ Lighting controls: Added L, [, ], P keyboard controls
7. ✅ Object positions: Updated paintings and furniture

**Current Status:** ~90% complete

---

*End of Changes Log*  
*Last Updated: April 26, 2026*