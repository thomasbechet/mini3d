use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum WGPURendererError {
    SurfaceAcquisition,
    MaxPassObjectReached,
    MaxVertexCountReached,
}

impl Error for WGPURendererError {}

impl Display for WGPURendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WGPURendererError::SurfaceAcquisition => write!(f, "SurfaceAcquisition"),
            WGPURendererError::MaxPassObjectReached => write!(f, "MaxPassObjectReached"),
            WGPURendererError::MaxVertexCountReached => write!(f, "MaxVertexCountReached"),
        }
    }
}