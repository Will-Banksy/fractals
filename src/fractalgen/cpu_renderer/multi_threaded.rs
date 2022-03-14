use super::super::{FractalType, PlaneTransform, ImageBufferFormat, PixelArrayFormat, ChannelArrayFormat};
use super::super::{common, julia, mandelbrot};
use image::RgbImage;
use rayon::prelude::*;

/// Generates an RGB image of the specified fractal, with given dimensions, and a defined transformation from the image coordinate plane to the complex plane, and the max_iterations is the amount of detail (50-100 being low, >=1000 being high, default 100)
pub fn generate_fractal_image(fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) -> RgbImage {
	let (width, height) = dimensions;

	let transform = transform.clone();

	let mut img_buffer: Vec<u8> = vec![0; width as usize * height as usize * 3];

	render_fractal_to(ImageBufferFormat::ChannelArray(ChannelArrayFormat::Rgb8(&mut img_buffer)), fractal_type, dimensions, &transform, max_iterations);

	RgbImage::from_raw(width, height, img_buffer).unwrap()
}

/// Generates an RGB image of the specified fractal, with given dimensions, and a defined transformation from the image coordinate plane to the complex plane, and the max_iterations is the amount of detail (50-100 being low, >=1000 being high, default 100)
///
/// The RGB image is written to the buffer contained within img_buffer_fmt, in the format specified
pub fn render_fractal_to(img_buffer_fmt: ImageBufferFormat, fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) {
	let (width, height) = dimensions;

	match img_buffer_fmt {
		ImageBufferFormat::PixelArray(px_arr_fmt) => match px_arr_fmt {
			PixelArrayFormat::Argb32(px_arr) => {
				let mut rows = common::into_rows_mut(px_arr, width, height);

				rows.par_iter_mut().enumerate().for_each(|(y, row)| {
					for x in 0..width {
						// let i = x as usize + y as usize * width as usize;

						let rgb = match fractal_type {
							FractalType::MandelbrotSet => mandelbrot::calculate_pixel(x, y as u32, transform, max_iterations),
							FractalType::JuliaSet => julia::calculate_pixel(x, y as u32, transform, max_iterations)
						};

						row[x as usize] = common::to_0rgb_u8(rgb.r as u8, rgb.g as u8, rgb.b as u8);
					}
				});
			}
		},
		ImageBufferFormat::ChannelArray(ch_arr_fmt) => match ch_arr_fmt {
			ChannelArrayFormat::Argb8(ch_arr) => {
				let mut rows = common::into_rows_mut(ch_arr, width * 4, height);

				rows.par_iter_mut().enumerate().for_each(|(y, row)| {
					let mut counter: u8 = 0;

					for x in 0..(width * 4) {
						if counter == 0 {
							let rgb = match fractal_type {
								FractalType::MandelbrotSet => mandelbrot::calculate_pixel(x / 4, y as u32, transform, max_iterations),
								FractalType::JuliaSet => julia::calculate_pixel(x / 4, y as u32, transform, max_iterations)
							};

							row[x as usize] = 255;
							row[x as usize + 1] = rgb.r as u8;
							row[x as usize + 2] = rgb.g as u8;
							row[x as usize + 3] = rgb.b as u8;

							counter = 3;
						} else {
							counter -= 1;
						}
					}
				});
			},
			ChannelArrayFormat::Rgb8(ch_arr) => {
				let mut rows = common::into_rows_mut(ch_arr, width * 3, height);

				rows.par_iter_mut().enumerate().for_each(|(y, row)| {
					let mut counter: u8 = 0;

					for x in 0..(width * 3) {
						if counter == 0 {
							let rgb = match fractal_type {
								FractalType::MandelbrotSet => mandelbrot::calculate_pixel(x / 3, y as u32, transform, max_iterations),
								FractalType::JuliaSet => julia::calculate_pixel(x / 3, y as u32, transform, max_iterations)
							};

							row[x as usize] = rgb.r as u8;
							row[x as usize + 1] = rgb.g as u8;
							row[x as usize + 2] = rgb.b as u8;

							counter = 2;
						} else {
							counter -= 1;
						}
					}
				});
			}
		}
	};
}