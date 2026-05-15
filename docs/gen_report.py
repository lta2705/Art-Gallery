#!/usr/bin/env python3
"""Generate the Final Project Report into Graphic Design.docx"""
import sys, io, os, copy
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

from docx import Document
from docx.shared import Pt, Inches, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml.ns import qn

INPUT = os.path.join(os.path.dirname(__file__), 'Graphic Design.docx')
OUTPUT = os.path.join(os.path.dirname(__file__), 'Graphic Design.docx')
BACKUP = os.path.join(os.path.dirname(__file__), 'Graphic Design_backup.docx')

doc = Document(INPUT)

# --- Backup ---
doc.save(BACKUP)
print(f"Backup saved to {BACKUP}")

# ============================================================
# HELPER FUNCTIONS
# ============================================================
def find_paragraph_index(doc, text_keyword):
    for i, p in enumerate(doc.paragraphs):
        if text_keyword in p.text:
            return i
    return -1

def clear_blank_paragraphs_after(doc, start_idx, end_idx):
    """Mark blank paragraphs for content insertion."""
    blanks = []
    for i in range(start_idx + 1, min(end_idx, len(doc.paragraphs))):
        if doc.paragraphs[i].text.strip() == '':
            blanks.append(i)
    return blanks

def add_heading_text(paragraph, text, size=14, bold=True):
    paragraph.clear()
    run = paragraph.add_run(text)
    run.bold = bold
    run.font.size = Pt(size)
    run.font.name = 'Times New Roman'

def add_body_text(paragraph, text, size=12, bold=False, italic=False):
    paragraph.clear()
    run = paragraph.add_run(text)
    run.bold = bold
    run.italic = italic
    run.font.size = Pt(size)
    run.font.name = 'Times New Roman'

def add_code_text(paragraph, text, size=9):
    paragraph.clear()
    paragraph.paragraph_format.left_indent = Inches(0.5)
    run = paragraph.add_run(text)
    run.font.size = Pt(size)
    run.font.name = 'Consolas'
    run.font.color.rgb = RGBColor(30, 30, 30)

def fill_section(doc, section_title, next_section_title, content_lines):
    """Fill blank paragraphs between section_title and next_section_title with content."""
    start = find_paragraph_index(doc, section_title)
    end = find_paragraph_index(doc, next_section_title) if next_section_title else len(doc.paragraphs)
    if start == -1:
        print(f"WARNING: Section '{section_title}' not found!")
        return
    blanks = clear_blank_paragraphs_after(doc, start, end)
    for i, line_info in enumerate(content_lines):
        if i >= len(blanks):
            break
        idx = blanks[i]
        p = doc.paragraphs[idx]
        text = line_info[0]
        fmt = line_info[1] if len(line_info) > 1 else 'body'
        if fmt == 'heading':
            add_heading_text(p, text, size=13)
        elif fmt == 'subheading':
            add_heading_text(p, text, size=12)
        elif fmt == 'code':
            add_code_text(p, text)
        elif fmt == 'italic':
            add_body_text(p, text, italic=True)
        elif fmt == 'bold':
            add_body_text(p, text, bold=True)
        else:
            add_body_text(p, text)

# ============================================================
# CHAPTER 1: INTRODUCTION
# ============================================================
ch1 = [
    ("1.1 Introduction to Project", "heading"),
    ("The Art Gallery is an interactive 3D computer graphics application that simulates a virtual art exhibition space. The project creates an L-shaped gallery environment where users navigate as a first-person observer, represented by a sphere (the \"head\"), exploring artworks displayed on walls while experiencing realistic lighting, shadows, and material interactions.",),
    ("The application is developed using the Rust programming language with the Bevy Engine (version 0.13), which employs a modern Entity-Component-System (ECS) architecture. Physics-based collision detection is handled by the Rapier3D library, and 3D models are loaded from standard STL and OBJ file formats.",),
    ("",),
    ("1.2 Motivation", "heading"),
    ("Traditional computer graphics courses often rely on legacy OpenGL with C/C++. This project explores a modern alternative: the Bevy Engine, which provides a Physically Based Rendering (PBR) pipeline out of the box, declarative ECS patterns, and GPU-accelerated rendering via the WGPU abstraction layer. The motivation is to demonstrate that fundamental graphics concepts (transformations, shading models, shadow mapping) remain consistent regardless of the rendering framework chosen.",),
    ("",),
    ("1.3 Aims and Objectives", "heading"),
    ("The primary objectives of this project are:", "bold"),
    ("(1) Construct an L-shaped 3D room from an STL mesh with accurate collision boundaries.",),
    ("(2) Implement texture mapping to display three distinct artworks on three separate walls.",),
    ("(3) Build a first-person navigation system using keyboard (WASD) and mouse input.",),
    ("(4) Create a dual-camera toggle system between First-Person View (FPV) and CCTV.",),
    ("(5) Implement a comprehensive lighting system with Point Lights, Spotlight, and soft shadows.",),
    ("(6) Apply anti-aliasing (4x MSAA) and shadow mapping for visual realism.",),
    ("",),
    ("1.4 Methodology", "heading"),
    ("The development follows a modular approach based on the ECS architecture. Each concern (geometry, physics, lighting, camera, input) is implemented as an independent Bevy System that operates on shared Components. The rendering pipeline uses Bevy's built-in PBR shaders, which implement the Cook-Torrance microfacet BRDF model for physically accurate material rendering.",),
    ("",),
    ("1.5 Report Outline", "heading"),
    ("Chapter 2 covers Geometry and Environment, including 3D modeling, asset loading, and texture mapping. Chapter 3 discusses Navigation and Viewports, analyzing coordinate spaces, the head-ball controller, and dual-camera logic. Chapter 4 (labeled Chapter 5 per syllabus) presents the Illumination, Shading, and Shadow systems, detailing the mathematical models behind point lights, spotlights, attenuation, and post-processing effects.",),
]

fill_section(doc, "CHAPTER 1: INTRODUCTION", "CHAPTER 2: GEOMETRY", ch1)

# ============================================================
# CHAPTER 2: GEOMETRY & ENVIRONMENT
# ============================================================
ch2 = [
    ("2.1 3D Modeling and Asset Loading", "heading"),
    ("",),
    ("2.1.1 Mesh Data Structure Theory", "subheading"),
    ("In computer graphics, a 3D object is represented as a polygon mesh: a collection of vertices (V), edges (E), and faces (F) satisfying Euler's formula V - E + F = 2 for closed manifolds. Each vertex stores a position vector P = (x, y, z), a normal vector N = (nx, ny, nz) perpendicular to the surface for lighting calculations, and optionally UV texture coordinates (u, v).",),
    ("The STL (Stereolithography) format stores geometry as an unstructured list of triangular facets. Each triangle is defined by three vertices and one face normal. The OBJ format additionally supports vertex normals, texture coordinates, and material references.",),
    ("",),
    ("2.1.2 Asset Loading via Bevy Asset Server", "subheading"),
    ("Bevy's AssetServer performs asynchronous loading of mesh data from disk to GPU memory. When a mesh handle is requested, the engine schedules an I/O task that parses the file, constructs vertex buffers, and uploads them to the GPU. The StlPlugin and ObjPlugin extend the default loader to support these formats.",),
    ("Code: Loading the L-shaped room from STL:", "italic"),
    ("commands.spawn(PbrBundle {", "code"),
    ("    mesh: asset_server.load(\"Models/art_gallery.stl\"),", "code"),
    ("    transform: Transform::from_xyz(0.0, 0.0, -6.5)", "code"),
    ("        .with_rotation(Quat::from_rotation_y(-PI/2) * Quat::from_rotation_x(-PI/2)),", "code"),
    ("    ..default()", "code"),
    ("});", "code"),
    ("",),
    ("2.1.3 Affine Transformations", "subheading"),
    ("To position the STL model correctly in world space, we apply affine transformations. The composite transformation matrix M = T * R * S transforms each vertex from model space to world space, where T is translation, R is rotation, and S is scaling.",),
    ("The STL file uses a Z-up coordinate system (CAD convention), while Bevy uses Y-up (right-handed). The rotation Quat::from_rotation_x(-PI/2) converts Z-up to Y-up by rotating -90 degrees around the X-axis.",),
    ("[Insert Figure 2.1: L-shaped room coordinate system and vertex mapping]",),
    ("",),
    ("2.2 Texture Mapping", "heading"),
    ("",),
    ("2.2.1 UV Mapping Theory", "subheading"),
    ("UV mapping is the process of projecting a 2D texture image onto a 3D surface. Each vertex is assigned coordinates (u, v) in texture space where u, v are in the range [0, 1]. During rasterization, the GPU interpolates UV coordinates across each triangle fragment and samples the corresponding texel from the texture image.",),
    ("For the gallery paintings, we use Bevy's built-in Rectangle mesh (a Quad), which provides default UV coordinates mapping the entire texture to the rectangular surface without distortion.",),
    ("",),
    ("2.2.2 Painting Implementation", "subheading"),
    ("Three paintings are placed on three different walls using the spawn_painting_with_frame() helper function. Each painting consists of a textured Quad (1.2m x 1.2m) surrounded by a procedural wooden frame made of four Cuboid meshes. The frame material uses a dark brown color (RGB: 0.2, 0.1, 0.05) with high roughness (0.7) to simulate wood.",),
    ("Art 1: Left wall at position (0.01, 1.5, -1.5), rotated 90 degrees to face inward.",),
    ("Art 2: Back wall at position (6.5, 1.5, -3.49), facing the main hallway.",),
    ("Art 3: Branch wall at position (1.75, 1.5, -6.49), facing the branch entrance.",),
    ("",),
    ("2.2.3 PBR Material Properties", "subheading"),
    ("Bevy's StandardMaterial implements the Cook-Torrance microfacet BRDF. Key parameters include: base_color (albedo), metallic (0.0 = dielectric, 1.0 = conductor), perceptual_roughness (surface micro-irregularity), and reflectance (Fresnel reflectance at normal incidence, default 0.5 = 4% reflection for dielectrics).",),
    ("The ceiling light fixtures use a glass-like material with specular_transmission = 1.0, ior = 1.5 (glass refraction index), and emissive color to simulate glowing from within.",),
]

fill_section(doc, "CHAPTER 2: GEOMETRY", "CHAPTER 3. NAVIGATION", ch2)

# ============================================================
# CHAPTER 3: NAVIGATION & VIEWPORTS
# ============================================================
ch3 = [
    ("3.1 Coordinate Spaces", "heading"),
    ("",),
    ("3.1.1 The Rendering Pipeline Coordinate Spaces", "subheading"),
    ("The graphics pipeline transforms vertices through a sequence of coordinate spaces:", "bold"),
    ("Model Space: Local coordinates as authored in the 3D modeling tool. The STL room vertices exist in this space.",),
    ("World Space: A shared global coordinate system. The Transform component positions entities here. Bevy uses a right-handed, Y-up system where +X is right, +Y is up, and -Z is forward.",),
    ("View Space (Camera Space): Coordinates relative to the camera. The view matrix V = inverse(camera_world_transform) translates the world so the camera sits at the origin looking down -Z.",),
    ("Clip Space: After applying the perspective projection matrix P, coordinates are in homogeneous clip space. The GPU performs perspective division (x/w, y/w, z/w) to obtain Normalized Device Coordinates (NDC).",),
    ("The complete vertex transformation is: V_clip = P * V * M * V_local",),
    ("",),
    ("3.1.2 Bevy Coordinate System", "subheading"),
    ("Bevy adopts a right-handed coordinate system with Y-up. This differs from CAD tools (typically Z-up) and requires a -90 degree rotation around X when importing STL models. The room dimensions in world space are: Main hallway 9.5m x 3.0m (X: 0 to 9.5, Z: -3 to 0), Branch 3.5m x 3.5m (X: 0 to 3.5, Z: -6.5 to -3), Ceiling height 3.2m.",),
    ("",),
    ("3.2 The Head Ball Controller", "heading"),
    ("",),
    ("3.2.1 Euler Angles and Quaternion Rotation", "subheading"),
    ("The player's orientation is controlled by mouse input that modifies Yaw (rotation around Y-axis) and Pitch (rotation around X-axis). Direct use of Euler angles can cause Gimbal Lock, a singularity where two rotation axes align and a degree of freedom is lost.",),
    ("Bevy uses Quaternions q = (w, x, y, z) internally to represent rotations. A quaternion encodes a rotation of angle theta around axis a as: q = cos(theta/2) + sin(theta/2) * (ax*i + ay*j + az*k). Quaternion multiplication composes rotations without singularities.",),
    ("",),
    ("3.2.2 Movement Vector Calculation", "subheading"),
    ("Player movement is relative to the facing direction. The input direction vector d = (dx, 0, dz) from WASD keys is rotated by the player's current quaternion: d_world = q * d * q_inverse. This ensures 'W' always moves forward regardless of where the player is looking.",),
    ("Code: Direction-relative movement:", "italic"),
    ("let dir = transform.rotation * dir.normalize();", "code"),
    ("velocity.linvel.x = dir.x * speed;", "code"),
    ("velocity.linvel.z = dir.z * speed;", "code"),
    ("The Y-axis velocity is preserved to allow gravity from the Rapier3D physics engine to pull the player downward naturally.",),
    ("",),
    ("3.2.3 Mouse Look Implementation", "subheading"),
    ("Mouse delta values are read from Bevy's MouseMotion events. Yaw rotation is applied to the Player entity (parent), while Pitch is applied to the FPV Camera entity (child). This separation prevents the pitch rotation from affecting the movement direction vector.",),
    ("Pitch is clamped to [-1.5, 1.5] radians (approximately +/-86 degrees) to prevent the camera from flipping upside down.",),
    ("",),
    ("3.3 Dual Camera Toggle (FPV and CCTV)", "heading"),
    ("",),
    ("3.3.1 Perspective Projection Matrix", "subheading"),
    ("Both cameras use perspective projection, which simulates how the human eye perceives depth. The perspective matrix maps the view frustum (defined by field-of-view, aspect ratio, near plane, and far plane) to the unit cube in NDC.",),
    ("The FPV camera is a child entity of the Player sphere, inheriting its world transform through Bevy's hierarchical transform propagation: M_final = M_player * M_camera_local. This means the camera automatically follows the player's position and yaw rotation.",),
    ("The CCTV camera is an independent entity fixed at position (4.75, 3.0, -1.5) near the ceiling, using the LookAt function: M_cctv = LookAt(eye=(4.75,3.0,-1.5), target=(4.75,0.0,-3.25), up=(0,1,0)).",),
    ("",),
    ("3.3.2 Toggle Mechanism", "subheading"),
    ("Pressing 'C' toggles the is_active boolean on both Camera components. Only one camera can be active at a time. The mouse_look system checks if the FPV camera is active before processing input, preventing phantom rotations while in CCTV mode.",),
]

fill_section(doc, "CHAPTER 3. NAVIGATION", "CHAPTER 5. ILLUMINATION", ch3)

# ============================================================
# CHAPTER 4 (5): ILLUMINATION & SHADING & SHADOW
# ============================================================
ch4 = [
    ("4.1 Point Light and Attenuation", "heading"),
    ("",),
    ("4.1.1 Point Light Theory", "subheading"),
    ("A point light emits radiance equally in all directions from a single position in world space. The irradiance (power per unit area) received at a surface point decreases with the square of the distance, following the inverse-square law: I(d) = I_0 / d^2, where I_0 is the source intensity and d is the distance from the light to the surface fragment.",),
    ("In Bevy, PointLight components specify intensity (in lumens), range (maximum distance), and radius (for soft shadow penumbra). The engine applies a windowing function to smoothly attenuate to zero at the specified range.",),
    ("",),
    ("4.1.2 Gallery Lighting Layout", "subheading"),
    ("Four point lights are distributed along the hallway ceiling at Y = 3.1m:", "bold"),
    ("Light 1: Position (1.5, 3.1, -1.5) - Near the entrance of the main hallway.",),
    ("Light 2: Position (4.75, 3.1, -1.5) - Center of the main hallway.",),
    ("Light 3: Position (8.0, 3.1, -1.5) - Far end of the main hallway.",),
    ("Light 4: Position (1.75, 3.1, -4.75) - Branch hallway, illuminating Art 3 and the Desk.",),
    ("Each light has intensity 7000 lumens, range 15m, and radius 0.15m for soft shadow edges. A 3D ceiling lamp model (eb_ceiling_light_01.obj) is attached as a child entity with emissive glass material.",),
    ("",),
    ("4.2 Spotlight and Cone Calculation", "heading"),
    ("",),
    ("4.2.1 Spotlight Cone Mathematics", "subheading"),
    ("A spotlight restricts illumination to a cone defined by inner angle (theta_i) and outer angle (theta_o). The intensity within the cone is calculated using the angular falloff factor: F = clamp((cos(alpha) - cos(theta_o)) / (cos(theta_i) - cos(theta_o)), 0, 1), where alpha is the angle between the light direction and the vector from the light to the fragment.",),
    ("The main gallery spotlight is positioned at (4.75, 3.2, -1.5) pointing straight down with inner_angle = 0.8 rad and outer_angle = 1.2 rad, creating a focused pool of light on the gallery floor.",),
    ("",),
    ("4.2.2 Dynamic Lighting Controls", "subheading"),
    ("Runtime keyboard controls allow adjustment of lighting parameters:", "bold"),
    ("Key L: Toggle spotlight on/off. Key P: Toggle all point lights on/off.",),
    ("Keys [ and ]: Decrease/increase spotlight intensity by 5% per frame.",),
    ("Arrow Left/Right: Simultaneously adjust both spotlight and point light intensity.",),
    ("Intensity is clamped between 0 and 200,000 lumens to prevent overflow.",),
    ("",),
    ("4.3 Shadow Mapping", "heading"),
    ("",),
    ("4.3.1 Shadow Mapping Theory", "subheading"),
    ("Shadow mapping is a two-pass rendering technique. In the first pass, the scene is rendered from the light's perspective into a depth buffer (shadow map). In the second pass, each fragment's position is projected into the light's space and compared against the shadow map. If the fragment's depth exceeds the stored depth, it is in shadow.",),
    ("Bevy implements Percentage-Closer Filtering (PCF) to produce soft shadow edges. The shadow map resolution is set to 1024x1024 pixels per light (reduced from the default 2048x2048) to improve performance: PointLightShadowMap { size: 1024 }.",),
    ("",),
    ("4.3.2 Distance-Based Shadow Optimization", "subheading"),
    ("To reduce GPU workload, the update_shadows_by_distance system dynamically enables shadows only for lights within 8 meters of the player. Lights beyond this threshold have shadows_enabled set to false, skipping their shadow pass entirely. This optimization reduces the number of active shadow maps from 4-5 to typically 1-2 per frame.",),
    ("",),
    ("4.4 Anti-Aliasing", "heading"),
    ("Multisample Anti-Aliasing (MSAA) with 4 samples is enabled via the Msaa::Sample4 resource. MSAA operates during rasterization by evaluating fragment coverage at 4 sub-pixel locations per pixel, then averaging the results. This smooths jagged edges on polygon boundaries without requiring post-processing.",),
    ("",),
    ("4.5 Physically Based Rendering (PBR) Shading Model", "heading"),
    ("Bevy's default shader implements the Cook-Torrance microfacet BRDF, which combines a diffuse term (Lambertian) with a specular term. The rendering equation is: L_o = integral(f_r * L_i * cos(theta) * d_omega), where f_r is the BRDF, L_i is incoming radiance, and theta is the angle between the surface normal and the light direction.",),
    ("The BRDF consists of: (1) Diffuse: f_diff = c_base / pi (Lambertian reflection), (2) Specular: f_spec = D * F * G / (4 * dot(N,L) * dot(N,V)), where D is the GGX normal distribution function, F is the Fresnel-Schlick approximation, and G is the Smith geometry function.",),
]

fill_section(doc, "CHAPTER 5. ILLUMINATION", None, ch4)

# ============================================================
# PROJECT STATEMENT
# ============================================================
ps = [
    ("Project Title: The Art Gallery - Interactive 3D Exhibition Space", "bold"),
    ("Theme: Room Layout Variation 3 (L-shaped room)", "bold"),
    ("",),
    ("This project develops an interactive 3D art gallery simulation using the Rust programming language and the Bevy Engine (v0.13). The application features an L-shaped exhibition room (9.5m x 6.5m, ceiling height 3.2m) loaded from an STL model, with three artworks displayed on three different walls.",),
    ("",),
    ("The user navigates the gallery as a first-person observer (represented by a sphere) using WASD keyboard controls and mouse look. A dual-camera system allows toggling between First-Person View (FPV) and a ceiling-mounted CCTV camera. The lighting system includes four ceiling-mounted point lights with 3D lamp models, a central spotlight with adjustable intensity, soft shadows, and 4x MSAA anti-aliasing.",),
    ("",),
    ("Key technologies: Rust, Bevy Engine 0.13, Rapier3D (physics/collision), bevy-stl and bevy-obj (3D model loading), WGPU (GPU abstraction layer), PBR (Physically Based Rendering) pipeline.",),
    ("",),
    ("Implemented features: L-shaped room from STL, 3 paintings with procedural frames, furniture (Desk_PC) with collision, point lights with distance-based shadow optimization, spotlight with keyboard controls (L/P/[/]), FPV-CCTV toggle (C key), WASD movement with mouse look, fall-reset safety system, reduced shadow map resolution for performance.",),
]

fill_section(doc, "PROJECT STATEMENT", "MEMBER CONTRIBUTION", ps)

# ============================================================
# NOMENCLATURES
# ============================================================
nom = [
    ("ECS - Entity-Component-System", "bold"),
    ("PBR - Physically Based Rendering",),
    ("BRDF - Bidirectional Reflectance Distribution Function",),
    ("FPV - First-Person View",),
    ("CCTV - Closed-Circuit Television (Security Camera)",),
    ("MSAA - Multisample Anti-Aliasing",),
    ("PCF - Percentage-Closer Filtering",),
    ("STL - Stereolithography (3D file format)",),
    ("OBJ - Wavefront Object (3D file format)",),
    ("UV - Texture coordinates (U=horizontal, V=vertical)",),
    ("NDC - Normalized Device Coordinates",),
    ("WGPU - WebGPU abstraction layer for cross-platform GPU access",),
    ("CCD - Continuous Collision Detection",),
    ("AABB - Axis-Aligned Bounding Box",),
    ("GGX - Ground Glass Unknown (microfacet distribution function)",),
    ("IOR - Index of Refraction",),
]

fill_section(doc, "NOMENCLATURES", "GLOSSARY", nom)

# ============================================================
# GLOSSARY
# ============================================================
glo = [
    ("Affine Transformation: A geometric transformation that preserves lines and parallelism, including translation, rotation, and scaling. Represented as a 4x4 matrix in homogeneous coordinates.",),
    ("Attenuation: The reduction in light intensity as distance from the source increases, typically following the inverse-square law.",),
    ("Collider: A physics shape used for collision detection. Types include TriMesh (triangulated mesh, precise but expensive) and Cuboid (axis-aligned box, fast approximation).",),
    ("Component: In ECS architecture, a data-only struct attached to an Entity. Examples: Transform, PointLight, Player.",),
    ("Entity: A unique identifier in the ECS world that groups Components together.",),
    ("Fragment Shader: A GPU program that computes the final color of each pixel (fragment) during rasterization.",),
    ("Gimbal Lock: A loss of one degree of rotational freedom when two axes align in an Euler angle representation.",),
    ("Quaternion: A four-dimensional number system (w, x, y, z) used to represent 3D rotations without gimbal lock.",),
    ("Shadow Map: A depth texture rendered from the light's perspective, used to determine which fragments are in shadow.",),
    ("System: In ECS architecture, a function that operates on entities matching a specific Component query each frame.",),
]

fill_section(doc, "GLOSSARY", "CHAPTER 1: INTRODUCTION", glo)

# ============================================================
# SAVE
# ============================================================
doc.save(OUTPUT)
print(f"Report written to {OUTPUT}")
print("Done!")
