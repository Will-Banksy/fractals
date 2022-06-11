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

	let extensions = InstanceExtensions {
		.. InstanceExtensions::none()
	};

	// let vk_instance = Instance::new(Default::default()).expect("Failed to create instance");

	let vk_instance = Instance::new(None, Version::V1_1, &extensions, None).expect("Failed to create Vulkan instance");

	let img = fractalgen::gpu_renderer::compute::generate_fractal_image(&vk_instance, FractalType::MandelbrotSet, dims, &transform, Some(50));
	img.save(format!("mandelbrot.png")).unwrap();
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