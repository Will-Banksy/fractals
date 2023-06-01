pub mod mandelbrot {
	vulkano_shaders::shader!{
		ty: "compute",
		path: "shaders/compute_mandelbrot.comp"
	}
}