pub mod vec2;
pub mod vec3;
pub mod vec4;

pub use vec2::*;
pub use vec3::*;
pub use vec4::*;

use super::fixed::{I32F16, I32F24, U32F16, U32F24};

pub type V2I32 = V2<i32>;
pub type V2U32 = V2<u32>;
pub type V2I32F16 = V2<I32F16>;
pub type V2U32F16 = V2<U32F16>;
pub type V2I32F24 = V2<I32F24>;
pub type V2U32F24 = V2<U32F24>;

pub type V3I32 = V3<i32>;
pub type V3U32 = V3<u32>;
pub type V3I32F16 = V3<I32F16>;
pub type V3U32F16 = V3<U32F16>;
pub type V3I32F24 = V3<I32F24>;
pub type V3U32F24 = V3<U32F24>;

pub type V4I32 = V4<i32>;
pub type V4U32 = V4<u32>;
pub type V4I32F16 = V4<I32F16>;
pub type V4U32F16 = V4<U32F16>;
pub type V4I32F24 = V4<I32F24>;
pub type V4U32F24 = V4<U32F24>;
