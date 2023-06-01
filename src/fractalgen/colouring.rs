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

pub fn map_to_colour_loop(iterations: f64, max_iterations: f64) -> Rgb {
	let n = iterations % max_iterations;
	map_to_colour(n, max_iterations)
}

pub fn normalised_iter_count(iterations: f64, max_iterations: f64) -> Rgb {
	todo!("get back broke ");
	todo!("https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Continuous_(smooth)_coloring")
}

pub fn map_to_purple(iterations: f64, max_iterations: f64) -> Rgb {
	if iterations == max_iterations {
		Rgb::new(0., 0., 0.)
	} else {
		let mapped_i = common::linear_map(iterations, 0., max_iterations, 0., 255.);
		Rgb::new(mapped_i, 0., mapped_i)
	}
}