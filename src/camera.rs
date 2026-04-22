//! Module B – Navigation & Viewports
//!
//! Quản lý Camera, ma trận View/Projection,
//! xử lý input WASD/Mouse, và chuyển đổi chế độ camera.

use glam::{Mat4, Vec3};

// ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum CameraMode {
    /// Chế độ người chơi: camera bám theo The Head
    POV,
    /// Chế độ giám sát: camera góc cố định
    CCTV,
}

pub struct Camera {
    // Vị trí và hướng HEAD (nhân vật)
    pub head_pos: Vec3,
    pub yaw: f32,   // radian, trục Y
    pub pitch: f32, // radian, trục X (giới hạn ±89°)

    // Cài đặt projection
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub mode: CameraMode,

    // CCTV: vị trí và target cố định
    pub cctv_pos: Vec3,
    pub cctv_target: Vec3,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            head_pos: Vec3::new(3.25, 0.5, -2.0),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            fov_y: 60_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100.0,
            mode: CameraMode::POV,
            cctv_pos: Vec3::new(3.25, 3.0, -0.5), // Góc phòng cao
            cctv_target: Vec3::new(3.25, 0.0, -5.0),
        }
    }

    /// Vector hướng nhìn của Head
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    /// Vector sang phải của Head
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// View Matrix dựa trên chế độ hiện tại
    pub fn view_matrix(&self) -> Mat4 {
        match self.mode {
            CameraMode::POV => {
                // Camera nhìn từ trong đầu Head về phía forward
                let eye = self.head_pos + Vec3::new(0.0, 0.3, 0.0); // mắt cao hơn tâm cầu
                let target = eye + self.forward();
                Mat4::look_at_rh(eye, target, Vec3::Y)
            }
            CameraMode::CCTV => Mat4::look_at_rh(self.cctv_pos, self.cctv_target, Vec3::Y),
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect, self.near, self.far)
    }

    /// Di chuyển Head (WASD/Arrow), delta_time để frame-rate independent
    pub fn move_head(&mut self, fwd: f32, right: f32, dt: f32) {
        const SPEED: f32 = 5.0;
        let f = self.forward() * Vec3::new(1.0, 0.0, 1.0); // chỉ di chuyển trên mặt phẳng XZ
        let r = self.right() * Vec3::new(1.0, 0.0, 1.0);
        self.head_pos += SPEED * dt * (f * fwd + r * right);
        self.head_pos.y = 0.5; // luôn trên sàn
    }

    /// Xử lý chuột: dx, dy là pixel offset
    pub fn rotate_mouse(&mut self, dx: f32, dy: f32) {
        const SENSITIVITY: f32 = 0.002;
        self.yaw += dx * SENSITIVITY;
        self.pitch =
            (self.pitch - dy * SENSITIVITY).clamp(-89_f32.to_radians(), 89_f32.to_radians());
    }

    /// Toggle CCTV ↔ POV (phím C)
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            CameraMode::POV => CameraMode::CCTV,
            CameraMode::CCTV => CameraMode::POV,
        };
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }
}

/// Trạng thái phím hiện tại – cập nhật trong winit event loop
#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl InputState {
    pub fn fwd_axis(&self) -> f32 {
        (self.forward as i32 - self.backward as i32) as f32
    }
    pub fn right_axis(&self) -> f32 {
        (self.right as i32 - self.left as i32) as f32
    }
}
