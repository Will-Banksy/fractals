use fractals::fractalgen::{self, FractalType, PlaneTransform, ImageBufferFormat, PixelArrayFormat};
use minifb::{Window, WindowOptions, Key, MouseButton, MouseMode};

fn main() {
	let (dims_presets, scale_presets, offset_presets) = (
		[
			(480u32, 360u32),
			(15360, 8640),
			(480, 360),
			(480, 360)
		],
		[
			(0.01, 0.01),
			(0.0005, 0.0005),
			(0.000000028662414792174364, 0.000000028662414792174364),
			(0.0000000012919080187191952, 0.0000000012919080187191952),
			(0.000000000000007678375638976293, 0.000000000000007678375638976293)
		],
		[
			(240., 180.),
			(7680., 4320.),
			(62327141.65736399, 213.97749196797176),
			(769919577.7859883, 230132211.5792833),
			(150615497492901.8, 39666261511409.945)
		]
	);

	let dims = dims_presets[0];
	let scale = scale_presets[0];
	let offset = offset_presets[0];
	let mut transform = PlaneTransform::new().scale(scale).base_offset(offset);
	let mut max_iters = 1000;

	let mut window = Window::new(
		"Fractals - ESC To Exit",
		dims.0 as usize,
		dims.1 as usize,
		WindowOptions::default()
	)
	.unwrap_or_else(|e| {
		panic!("{}", e);
	});

	// ARGB array
	let mut buffer: Vec<u32> = vec![0; dims.0 as usize * dims.1 as usize];

	// Limit to ~20fps
	window.limit_update_rate(Some(std::time::Duration::from_secs_f32(0.05)));

	// When this is true, we rerender the fractal
	let mut rerender = true;

	while window.is_open() && !window.is_key_down(Key::Escape) {
		if window.get_mouse_down(MouseButton::Right) {
			transform.scale_x *= 1.2;
			transform.scale_y *= 1.2;

			match window.get_mouse_pos(MouseMode::Pass) {
				None => (),
				Some(mpos) => {
					// let mpos = (mpos.0 as f64, mpos.1 as f64);
					// offset_x = mpos.0 - ((mpos.0 - offset_x) * 1.2);
					// offset_y = mpos.1 - ((mpos.1 - offset_y) * 1.2);
					transform.base_offset_x = lerp(transform.base_offset_x, mpos.0 as f64, /*0.2*/5./30.);
					transform.base_offset_y = lerp(transform.base_offset_y, mpos.1 as f64, /*0.2*/5./30.);
				}
			}

			rerender = true;
		}

		if window.get_mouse_down(MouseButton::Left) {
			transform.scale_x /= 1.2;
			transform.scale_y /= 1.2;

			match window.get_mouse_pos(MouseMode::Pass) {
				None => (),
				Some(mpos) => {
					// let mpos = (mpos.0 as f64, mpos.1 as f64);
					// offset_x = mpos.0 - ((mpos.0 - offset_x) / 1.2);
					// offset_y = mpos.1 - ((mpos.1 - offset_y) / 1.2);
					transform.base_offset_x = lerp(transform.base_offset_x, mpos.0 as f64, -0.2);
					transform.base_offset_y = lerp(transform.base_offset_y, mpos.1 as f64, -0.2);
				}
			}

			rerender = true;
		}

		window.get_keys().iter().for_each(|key| {
			match key {
				Key::X => {
					max_iters = (max_iters as f64 * 1.1) as u32;
					*&mut rerender = true;
				},
				Key::Z => {
					max_iters = (max_iters as f64 / 1.1) as u32;
					*&mut rerender = true;
				},
				_ => ()
			}
		});

		if rerender {
			fractalgen::cpu_renderer::multi_threaded::render_fractal_to(ImageBufferFormat::PixelArray(PixelArrayFormat::Argb32(&mut buffer)), FractalType::MandelbrotSet, dims, &transform, Some(max_iters));
			rerender = false;
		}

		window.update_with_buffer(&buffer, dims.0 as usize, dims.1 as usize).unwrap();
	}

	println!("Scale: ({}, {}), Offset: ({}, {})", transform.scale_x, transform.scale_y, transform.base_offset_x, transform.base_offset_y);
}

fn linear_map(value: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64{
	start2 + (stop2 - start2) * ((value - start1) / (stop1 - start1))
}

fn lerp(value: f64, target: f64, amt: f64) -> f64 {
	linear_map(amt, 0., 1., value, target)
}