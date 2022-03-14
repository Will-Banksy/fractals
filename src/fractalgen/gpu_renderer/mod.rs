//! This module uses Vulkan (via the vulkano crate) to calculate/render fractals

// https://vulkano.rs/guide/initialization

/// This module uses compute shaders to calculate the fractal image
pub mod compute;
/// This module uses graphics shaders to render the fractal image to a context
pub mod render;
