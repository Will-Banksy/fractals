//! This module contains the functions `generate_fractal_image` and `render_fractal_to`, allowing to specify using an enum whether to render them using multiple threads or one
//!
//! Alternatively, it also contains the modules `single_threaded` and `multi_threaded` with those same functions, using one thread or multiple respectively
//!
//! These calculations are all done exclusively on the CPU

/// This module contains the singlethreaded variants of the functions `generate_fractal_image` and `render_fractal_to`
pub mod single_threaded;
/// This module contains the multithreaded variants of the functions `generate_fractal_image` and `render_fractal_to`
pub mod multi_threaded;

use super::{FractalType, PlaneTransform, ImageBufferFormat};
use image::RgbImage;

pub enum Threadedness {
	Singlethreaded,
	Multithreaded
}

/// Generates an RGB image of the specified fractal, with given dimensions, and a defined transformation from the image coordinate plane to the complex plane, and the max_iterations is the amount of detail (50-100 being low, >=1000 being high, default 100)
///
/// Specify using `threadedness` whether to calculate using one thread or multiple
pub fn generate_fractal_image(threadedness: Threadedness, fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) -> RgbImage {
	match threadedness {
		Threadedness::Singlethreaded => single_threaded::generate_fractal_image(fractal_type, dimensions, transform, max_iterations),
		Threadedness::Multithreaded => multi_threaded::generate_fractal_image(fractal_type, dimensions, transform, max_iterations)
	}
}

/// Generates an RGB image of the specified fractal, with given dimensions, and a defined transformation from the image coordinate plane to the complex plane, and the max_iterations is the amount of detail (50-100 being low, >=1000 being high, default 100)
///
/// The RGB image is written to the buffer contained within img_buffer_fmt, in the format specified
///
/// Specify using `threadedness` whether to calculate using one thread or multiple
pub fn render_fractal_to(threadedness: Threadedness, img_buffer_fmt: ImageBufferFormat, fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) {
	match threadedness {
		Threadedness::Singlethreaded => single_threaded::render_fractal_to(img_buffer_fmt, fractal_type, dimensions, transform, max_iterations),
		Threadedness::Multithreaded => multi_threaded::render_fractal_to(img_buffer_fmt, fractal_type, dimensions, transform, max_iterations)
	}
}