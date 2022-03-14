pub mod shaders;

use vulkano::instance::{Instance};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};

use super::super::super::fractalgen::{PlaneTransform, FractalType, ImageBufferFormat};
use std::sync::Arc;

pub struct VulkanCompute {
	physical: PhysicalDevice<'static>,
	queue_family: QueueFamily<'static>,
	device: Arc<Device>,
	queue: Arc<Queue>
}

impl VulkanCompute {
	pub fn new(vk_instance: &'static Arc<Instance>) -> Self {
		let device_extensions = DeviceExtensions {
			..DeviceExtensions::none()
		};

		let (physical, queue_family) = VulkanCompute::select_compute_device(&vk_instance, &device_extensions);

		println!("Using vulkan device: {} (type: {:?})", physical.properties().device_name, physical.properties().device_type);

		let (device, mut queues) = Device::new(
			physical,
			&Features::none(),
			&physical.required_extensions().union(&device_extensions),
			[(queue_family, 0.5)].iter().cloned()
		).expect("Failed to create a device");

		let queue = queues.next().unwrap();

		VulkanCompute {
			physical,
			queue_family,
			device,
			queue
		}
	}

	pub fn select_compute_device(instance: &'static Arc<Instance>, device_extensions: &DeviceExtensions) -> (PhysicalDevice<'static>, QueueFamily<'static>) {
		PhysicalDevice::enumerate(&instance)
			.filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
			.filter_map(|p| {
				// The Vulkan specs guarantee that a compliant implementation must provide at least one queue that supports compute operations
				p.queue_families()
					.find(|&q| q.supports_compute())
					.map(|q| (p, q))
			})
			.min_by_key(|(p, _)| match p.properties().device_type {
				PhysicalDeviceType::DiscreteGpu => 0,
				PhysicalDeviceType::IntegratedGpu => 1,
				PhysicalDeviceType::VirtualGpu => 2,
				PhysicalDeviceType::Cpu => 3,
				PhysicalDeviceType::Other => 4
			}).expect("No vulkan implementations found")
	}

	pub fn render_fractal_to(&self, img_buffer_fmt: ImageBufferFormat, fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) {
		println!("Starting Vulkan stuff...");

		let (width, height) = dimensions;

		if true { // Render mandelbrot example
			// Create data buffer to be operated on
			let data_dat = (0..(width * height)).map(|_| 0);
			let data_buf = CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), false, data_dat).expect("Failed to create buffer");

			let shader = shaders::mandelbrot::compute::load(self.device.clone()).expect("Failed to create shader module");

			let compute_pipeline = ComputePipeline::new(
				self.device.clone(),
				shader.entry_point("main").unwrap(),
				&(),
				None,
				|_| {}
			).expect("Failed to create compute pipeline");

			let layout = compute_pipeline
				.layout()
				.descriptor_set_layouts()
				.get(0)
				.unwrap();

			let set = PersistentDescriptorSet::new(layout.clone(), [WriteDescriptorSet::buffer(0, data_buf.clone())]).unwrap();

			let mut builder = AutoCommandBufferBuilder::primary(self.device.clone(), self.queue.family(), CommandBufferUsage::OneTimeSubmit).unwrap();

			builder.bind_pipeline_compute(compute_pipeline.clone())
				.bind_descriptor_sets(
					PipelineBindPoint::Compute,
					compute_pipeline.layout().clone(),
					0,
					set
				)
				.dispatch([1024, 1, 1])
				.unwrap();

			let command_buffer = builder.build().unwrap();

			let future = sync::now(self.device.clone())
				.then_execute(self.queue.clone(), command_buffer)
				.unwrap()
				.then_signal_fence_and_flush()
				.unwrap();

			future.wait(None).unwrap();

			let content = data_buf.read().unwrap();
			for n in content.iter() {
				assert_eq!(*n, 0xffffffu32);
			}
		}

		println!("Finished Vulkan stuff successfully");
	}

// pub fn render_mandelbrot_window() {
// 	unimplemented!();
// }
}