use mini3d::{graphics::{SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_ASPECT_RATIO}, glam::{Vec4, Vec4Swizzles}};

pub fn compute_fixed_viewport(viewport: Vec4) -> Vec4 {
    let pos = viewport.xy();
    let size = viewport.zw();
    if size.x / size.y >= SCREEN_ASPECT_RATIO {
        let w = size.y * SCREEN_ASPECT_RATIO;
        let h = size.y;
        let x = (size.x / 2.0) - (w / 2.0);
        let y = 0.0;
        (pos.x + x, pos.y + y, w, h).into()
    } else {
        let w = size.x;
        let h = SCREEN_HEIGHT as f32 * size.x / SCREEN_WIDTH as f32;
        let x = 0.0;
        let y = (size.y / 2.0) - (h / 2.0);
        (pos.x + x, pos.y + y, w, h).into()
    }
}