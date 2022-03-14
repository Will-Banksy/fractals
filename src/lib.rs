/// Base module for the library for generating fractals
pub mod fractalgen {
	pub mod common;
	pub mod mandelbrot;
	pub mod julia;
	pub mod cpu_renderer;
	pub mod gpu_renderer;
	pub mod colouring;

	// Rexport fractalgen::common::PlaneTransform as fractalgen::PlaneTransform
	pub use common::{PlaneTransform};

	#[derive(Clone, Copy)]
	pub enum FractalType {
		MandelbrotSet,
		JuliaSet
	}

	/// Specifies the format of and holds a mutable reference to either a pixel array or channel array
	pub enum ImageBufferFormat<'a> {
		PixelArray(PixelArrayFormat<'a>),
		ChannelArray(ChannelArrayFormat<'a>)
	}

	/// Specifies the format of and holds a mutable reference to a pixel array
	pub enum PixelArrayFormat<'a> {
		Argb32(&'a mut [u32]),
	}

	/// Specifies the format of and holds a mutable reference to a channel array
	pub enum ChannelArrayFormat<'a> {
		Argb8(&'a mut [u8]),
		Rgb8(&'a mut [u8])
	}
}