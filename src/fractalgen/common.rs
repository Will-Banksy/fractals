//! Contains a few useful functions and most importantly defines the `PlaneTransform` struct

use std::ops::{AddAssign, SubAssign, Mul, MulAssign, Div};
use num::{Zero, One};

/// Defines a transform from a base plane to a transformed plane
#[derive(Clone, Copy)]
pub struct PlaneTransform<T> where T: Clone + Copy + One + Zero + PartialOrd + MulAssign + AddAssign + SubAssign + Div<f64, Output = T> + Mul {
	pub scale_x: T,
	pub scale_y: T,
	pub base_offset_x: T,
	pub base_offset_y: T,
	pub transformed_offset_x: T,
	pub transformed_offset_y: T
}

impl<T> PlaneTransform<T> where T: Clone + Copy + One + Zero + PartialOrd + MulAssign + AddAssign + SubAssign + Div<f64, Output = T> + Mul {
	pub fn new() -> Self {
		PlaneTransform {
			scale_x: One::one(),
			scale_y: One::one(),
			base_offset_x: Zero::zero(),
			base_offset_y: Zero::zero(),
			transformed_offset_x: Zero::zero(),
			transformed_offset_y: Zero::zero()
		}
	}

	/// Builder-style method
	pub fn scale(mut self, scale: (T, T)) -> Self {
		self.scale_x *= scale.0;
		self.scale_y *= scale.1;

		self
	}

	/// Builder-style method
	///
	/// Offsets the origin in the base plane
	pub fn base_offset(mut self, offset: (T, T)) -> Self {
		self.base_offset_x += offset.0;
		self.base_offset_y += offset.1;

		self
	}

	/// Builder-style method
	///
	/// Offsets the origin in the transformed plane
	pub fn transformed_offset(mut self, offset: (T, T)) -> Self {
		self.transformed_offset_x += offset.0;
		self.transformed_offset_y += offset.1;

		self
	}

	/// Builder-style method
	///
	/// Centres the origin inside base_dimensions (which are dimensions in the base plane). May change either of/both base and transformed offset
	pub fn centre(mut self, base_dimensions: (T, T)) -> Self {
		// If scale is greater than or equal to 1, then use the base offset as that will be more/as accurate. Otherwise, use transformed offset as that will be more accurate
		if self.scale_x >= One::one() {
			self.base_offset_x = base_dimensions.0 / 2.; // FIXME potential integer overflow problems here
		} else {
			self.transformed_offset_x = (base_dimensions.0 / 2.) * self.scale_x;
		}

		if self.scale_y >= One::one() {
			self.base_offset_y = base_dimensions.1 / 2.;
		} else {
			self.transformed_offset_y = (base_dimensions.1 / 2.) * self.scale_y;
		}

		self
	}

	pub fn transform(&self, coords: (T, T)) -> (T, T) {
		let (mut x, mut y) = coords;
		x -= self.base_offset_x;
		y -= self.base_offset_y;
		let mut tx = x * self.scale_x;
		let mut ty = y * self.scale_y;
		tx -= self.transformed_offset_x;
		ty -= self.transformed_offset_y;

		(tx, ty)
	}
}

pub fn linear_map(value: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
	start2 + (stop2 - start2) * ((value - start1) / (stop1 - start1))
}

// fn expo_out_map(value: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
// 	let x = linear_map(value, start1, stop1, 0., 1.);
// 	let normalised = if x == 1. { 1. } else { 1. - 2f64.powf(-10. * x) };
// 	linear_map(normalised, 0., 1., start2, stop2)
// }

pub fn to_0rgb_u8(r: u8, g: u8, b: u8) -> u32 {
	((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn into_rows_mut<T>(img_buffer: &mut [T], width: u32, height: u32) -> Vec<&mut [T]> {
	let mut rows: Vec<&mut [T]> = Vec::with_capacity(height as usize);
	let (part0, mut part1) = img_buffer.split_at_mut(width as usize);
	rows.push(part0);
	for _ in 0..(height - 1) {
		let (p0, p1) = part1.split_at_mut(width as usize);
		part1 = p1;
		rows.push(p0);
	}
	rows
}