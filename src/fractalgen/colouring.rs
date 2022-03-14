use color_space::{Rgb, Hsv};
use super::common;

pub fn map_to_colour(iterations: f64, max_iterations: f64) -> Rgb {
	if iterations == max_iterations {
		Rgb::new(0., 0., 0.)
	} else {
		let hue = common::linear_map(iterations, 0., max_iterations, 0., 360.);

		let col = Hsv::new(hue, 1., 1.); // Hsv colour
		Rgb::from(col) // Convert to rgb
	}
}

pub fn ethan(iterations: f64, max_iterations: f64) -> Rgb {
	if iterations == max_iterations {
		Rgb::new(0., 0., 0.)
	} else {
		let mapped_i = common::linear_map(iterations, 0., max_iterations, 0., 255.);
		Rgb::new(mapped_i, 0., mapped_i)
	}
}