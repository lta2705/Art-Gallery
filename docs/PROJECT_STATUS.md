# Project Status: 3D Art Gallery (Bevy Migration)

## 1. Overview
Dự án là một ứng dụng triển lãm nghệ thuật 3D, được chuyển đổi từ nền tảng OpenGL thuần sang **Bevy Engine 0.13**. Dự án tích hợp vật lý **Rapier3D** để xử lý va chạm và hệ thống **ECS** để quản lý thực thể (tranh, đèn, nội thất).

- **Tech Stack**: Rust, Bevy 0.13, Rapier3D, Bevy-STL, Bevy-OBJ.
- **Không gian**: Phòng chữ L kích thước 9.5m x 6.5m, trần cao 3.2m.

## 2. Các thay đổi đã thực hiện (Completed)

### 🏗️ Kiến trúc & Vật lý
- **Migration**: Chuyển đổi thành công toàn bộ logic hiển thị và vòng lặp sự kiện sang Bevy.
- **Collision Fix**: Khắc phục lỗi rơi tự do bằng cách lót một mặt sàn cố định (Manual Floor Collider) và bật tính năng **CCD** cho người chơi.
- **Mesh Analysis**: Tích hợp module `read_bin` để phân tích kích thước thực tế của các model 3D (`STL`, `OBJ`) ngay khi nạp, giúp căn chỉnh tỷ lệ (scale) chính xác.

### 💡 Ánh sáng & Đồ họa
- **Hệ thống đèn**: 
    - Phân bổ 3 đèn trần rải đều dọc hành lang chính.
    - Tích hợp model 3D cho chao đèn (`eb_ceiling_light_01.obj`) và bóng đèn phát sáng (emissive).
- **Realistic Shadows**: Kích hoạt **Soft Shadows** (bóng đổ mềm) với thông số `radius` vật lý, giảm `AmbientLight` để tăng độ tương phản.
- **Điều khiển**: Hỗ trợ phím tắt `L` (Spotlight), `P` (Pointlight), `[` / `]` (Cường độ sáng).

### 🖼️ Nội thất & Trang trí
- **Khung tranh**: Triển khai hệ thống bọc tranh tự động bằng khung gỗ trang trí (procedural frames).
- **Bố cục**: Đưa bàn `Desk_PC` về đúng góc `p6` và căn chỉnh tỷ lệ phù hợp với căn phòng.
- **Vị trí**: Đã fix lỗi Art 2 không áp sát tường (lùi 0.1m).

## 3. Các việc cần làm & Cải tiến (Backlog)

### 🛠️ Kỹ thuật & Hiệu năng
- **Fix STL Floor**: Kiểm tra lại mặt lưới sàn của file `art_gallery.stl` để có thể sử dụng trực tiếp TriMesh collider thay vì dùng sàn Box giả.
- **Decoration Struct**: Nâng cấp hệ thống `spawn_painting_with_frame` để hỗ trợ nạp model 3D khung tranh bên ngoài thay vì dùng khối hộp đơn giản.

### ✨ Trải nghiệm & Thẩm mỹ
- **Nội thất**: Bổ sung thêm các vật dụng trang trí khác (ghế, bục trưng bày) để không gian bớt trống trải.
- **Hậu kỳ (Post-processing)**: Thêm hiệu ứng **Bloom** (để đèn trông lung linh hơn) và **Ambient Occlusion** (để các góc tường trông có chiều sâu hơn).
- **Interactivity**: Thêm tính năng tương tác (ví dụ: nhấn phím để xem thông tin chi tiết về bức tranh).

---
*Cập nhật lần cuối: 14/05/2026*
