use fractals::fractalgen::{self, FractalType};
use fractalgen::gpu_renderer;

use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::image::{ImageUsage};
use vulkano::render_pass::RenderPass;

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder, Window};
use winit::event::{Event, WindowEvent};

use fractalgen::PlaneTransform;
use std::sync::Arc;

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

	// let img = fractalgen::gpu_renderer::render_fractal_to(FractalType::MandelbrotSet, dims, &transform, Some(50));
	// img.save(format!("mandelbrot.png")).unwrap();
	panic!();

	let required_exts = vulkano_win::required_extensions();
	let instance = Instance::new(None, Version::V1_1, &required_exts, None).unwrap();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new();
	let surface = window.build_vk_surface(&event_loop, instance.clone()).unwrap();

	let device_exts = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::none()
	};

	let (physical_device, queue_family) = select_physical_graphics_device(&instance, surface.clone(), &device_exts);

	let (device, mut queues) = {
		Device::new(
			physical_device,
			&Features::none(),
			&physical_device.required_extensions().union(&device_exts),
			[(queue_family, 0.5)].iter().cloned()
		).expect("Failed to create device")
	};

	let queue = queues.next().unwrap();

	let caps = surface.capabilities(physical_device).expect("Failed to get surface capabilities");

	let dimensions: [u32; 2] = surface.window().inner_size().into();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = caps.supported_formats[0].0;

	let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
		.num_images(caps.min_image_count + 1) // How many buffers to use in the swapchain. It's good to have at least one more than the min, to give more freedom to image queue
		.format(format)
		.dimensions(dimensions)
		.usage(ImageUsage::color_attachment()) // What the images are going to be used for
		.sharing_mode(&queue) // The queue(s) that the resource will be used (?)
		.composite_alpha(composite_alpha) // How alpha values will be treated
		.build()
		.expect("Failed to create swapchain");

	let render_pass = get_render_pass(device.clone(), swapchain.clone());

	unimplemented!();

	// event_loop.run(move |event, _, control_flow| {
	// 	*control_flow = ControlFlow::Poll;

	// 	match event {
	// 		Event::WindowEvent {
	// 			event: WindowEvent::CloseRequested,
	// 			..
	// 		} => {
	// 			println!("Exiting...");
	// 			*control_flow = ControlFlow::Exit;
	// 		},
	// 		Event::MainEventsCleared => {
	// 			// Decide whether redraw is necessary

	// 			// If it is, request redraw
	// 			window.request_redraw();
	// 		},
	// 		Event::RedrawRequested(_) => {
	// 			// Redraw
	// 		},
	// 		_ => ()
	// 	}
	// });
}

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Arc<RenderPass> {
	vulkano::single_pass_renderpass!(
		device.clone(),
		attachments: {
			color: {
				load: Clear,
				store: Store,
				format: swapchain.format(), // Use same format as swapchain
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {}
		}
	).unwrap()
}

fn select_physical_graphics_device<'a>(
	instance: &'a Arc<Instance>, surface: Arc<Surface<Window>>, device_extensions: &DeviceExtensions
) -> (PhysicalDevice<'a>, QueueFamily<'a>) {
	let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
		.filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
		.filter_map(|p| {
			p.queue_families()
				.find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
				.map(|q| (p, q))
		})
		.min_by_key(|(p, _)| match p.properties().device_type {
			PhysicalDeviceType::DiscreteGpu => 0,
			PhysicalDeviceType::IntegratedGpu => 1,
			PhysicalDeviceType::VirtualGpu => 2,
			PhysicalDeviceType::Cpu => 3,
			PhysicalDeviceType::Other => 4
		})
		.expect("No devices available");

	(physical_device, queue_family)
}