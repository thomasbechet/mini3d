use mini3d_core::{
    glam::{Vec4, Vec4Swizzles},
    renderer::{SCREEN_ASPECT_RATIO, SCREEN_HEIGHT, SCREEN_INV_ASPECT_RATIO, SCREEN_WIDTH},
};

#[derive(Debug, Clone, Copy)]
pub enum ViewportMode {
    Fixed(f32),
    FixedBestFit,
    StretchKeepAspect,
    Stretch,
}

pub fn compute_fixed_viewport(global_viewport: Vec4, mode: ViewportMode) -> Vec4 {
    let global_pos = global_viewport.xy().floor();
    let global_size = global_viewport.zw().floor();

    let size = match mode {
        ViewportMode::Fixed(factor) => {
            (factor * SCREEN_WIDTH as f32, factor * SCREEN_HEIGHT as f32)
        }
        ViewportMode::FixedBestFit => {
            let w_factor = global_size.x / SCREEN_WIDTH as f32;
            let h_factor = global_size.y / SCREEN_HEIGHT as f32;
            let min = f32::floor(w_factor.min(h_factor)).max(1.0);
            (min * SCREEN_WIDTH as f32, min * SCREEN_HEIGHT as f32)
        }
        ViewportMode::StretchKeepAspect => {
            if global_size.x / global_size.y >= SCREEN_ASPECT_RATIO {
                let w = global_size.y * SCREEN_ASPECT_RATIO;
                let h = global_size.y;
                (w.floor(), h.floor())
            } else {
                let w = global_size.x;
                let h = global_size.x * SCREEN_INV_ASPECT_RATIO;
                (w.floor(), h.floor())
            }
        }
        ViewportMode::Stretch => (global_size.x, global_size.y),
    };

    let x = (global_size.x / 2.0) - (size.0 / 2.0);
    let y = (global_size.y / 2.0) - (size.1 / 2.0);
    (global_pos.x + x, global_pos.y + y, size.0, size.1).into()
}
