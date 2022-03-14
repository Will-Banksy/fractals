use fractals::fractalgen::{self, FractalType, PlaneTransform};

fn main() {
	#[allow(unused)]
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

	let dims = (1920, 1200);//dims_presets[1];
	let scale = (0.002, 0.002);//scale_presets[1];
	let offset = (960., 600.);//offset_presets[1];
	let transform = PlaneTransform::new().scale(scale).base_offset(offset);

	let img = fractalgen::cpu_renderer::multi_threaded::generate_fractal_image(FractalType::MandelbrotSet, dims, &transform, Some(50));
	img.save(format!("mandelbrot.png")).unwrap();
}