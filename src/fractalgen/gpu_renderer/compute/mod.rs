pub mod shaders;

use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder, Window};
use winit::event::{Event, WindowEvent};
use vulkano::swapchain::Surface;

use super::super::super::fractalgen::{PlaneTransform, FractalType, ImageBufferFormat};
use std::sync::Arc;

pub struct VulkanCompute {
}

impl VulkanCompute {
	pub fn render_fractal_to(img_buffer_fmt: ImageBufferFormat, fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) {
		println!("Starting Vulkan stuff...");

		let (width, height) = dimensions;

		// Create a vulkan instance
		let vk_instance = Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None).expect("Failed to create vulkan instance");

		// Get the first vulkan device
		let physical = PhysicalDevice::enumerate(&vk_instance).next().expect("No devices supporting vulkan available");

		// Find a queue family that supports graphics
		let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).expect("Couldn't find a graphics queue family");

		let (device, mut queues) = Device::new(physical, &Features::none(), &DeviceExtensions::none(), [(queue_family, 0.5)].iter().cloned()).expect("Failed to create device");

		let queue = queues.next().unwrap();

		// let data = [0; 128];
		// let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data).expect("Failed to create buffer"); // device.clone clones the Arc

		if false { // Copy buffer example
			let source_dat = 0..64; // Iterator that produces 64 values of 0 to 63
			let source_buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, source_dat).expect("Failed to create buffer");

			let dest_dat = (0..64).map(|_| 0); // Iterator for 64 0s
			let dest_buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, dest_dat).expect("Failed to create buffer");

			// Create a primary command buffer builder
			let mut com_buf_builder = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::OneTimeSubmit).expect("Failed to create command buffer builder");

			// Add the copy buffer command to command buffer builder
			com_buf_builder.copy_buffer(source_buf.clone(), dest_buf.clone()).expect("Failed to add command to command buffer builder");

			// Turn the command buffer builder into an actual command buffer
			let command_buffer = com_buf_builder.build().expect("Failed to create command buffer");

			let future = sync::now(device.clone())
				.then_execute(queue.clone(), command_buffer)
				.unwrap()
				.then_signal_fence_and_flush() // Same as signal fence followed by flush
				.expect("Flush error");

			future.wait(None).unwrap(); // None is optional timeout

			let source_content = source_buf.read().expect("Failed to read buffer");
			let dest_content = dest_buf.read().expect("Failed to read buffer");
			assert_eq!(&*source_content, &*dest_content);
		}

		if true { // Render mandelbrot example
			// Create data buffer to be operated on
			let data_dat = (0..(width * height)).map(|_| 0);
			let data_buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_dat).expect("Failed to create buffer");

			let shader = shaders::mandelbrot::compute::load(device.clone()).expect("Failed to create shader module");

			let compute_pipeline = ComputePipeline::new(
				device.clone(),
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

			let mut builder = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::OneTimeSubmit).unwrap();

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

			let future = sync::now(device.clone())
				.then_execute(queue.clone(), command_buffer)
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