//! This module contains the function `calculate_pixel` to calculate the colour of the pixel at (x, y) for the mandelbrot set by transforming it into a coordinate in the complex plane using a defined transformation
//!
//! For colouring rules, it linearly maps the number of iterations taken for z to escape to hue in the HSV/HSB colour space, and if it doesn't escape then it returns black

use super::common::PlaneTransform;
use color_space::Rgb;
use super::colouring;

pub fn calculate_pixel(x: u32, y: u32, transform: &PlaneTransform<f64>, max_iterations: Option<u32>) -> Rgb {
	let max_iterations = max_iterations.unwrap_or(100);

	let (cx, cy) = transform.transform((x as f64, y as f64));

	let c = num::Complex::new(cx, cy);
	let mut z = num::Complex::new(0., 0.);

	let mut i = 0;
	while i < max_iterations && z.norm_sqr() <= 4. { // If z increases beyond 2, then it is not in the mandelbrot set
		z = z * z + c;
		i += 1;
	}

	colouring::map_to_colour(i as f64, max_iterations as f64)
}