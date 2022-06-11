//! A very simple abstraction over or helper with Vulkan compute that is only suited to doing basic things

use vulkano::Version;
use vulkano::image::StorageImage;
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::shader::ShaderModule;

use std::sync::Arc;

trait VkCommand { // TODO: This needs to be updated to have descriptor sets and other things as parameters and stuff
	fn add_to_buffer<T>(command_buffer_builder: &mut AutoCommandBufferBuilder<T>);
}

struct VkInstance {
	pub instance: Arc<Instance>
}

struct VkTarget<'a> {
	pub physical: PhysicalDevice<'a>,
	pub queue_family: QueueFamily<'a>,
	pub device: Arc<Device>,
	pub queue: Arc<Queue>,
}

struct VkComputeOperation<'a> {
	vk_target: &'a VkTarget<'a>,
	command_buffer: Arc<PrimaryAutoCommandBuffer>
}

enum VkDataStorage {
	Image(Arc<StorageImage>),
	BufferU8(Arc<CpuAccessibleBuffer<[u8]>>),
	BufferU32(Arc<CpuAccessibleBuffer<[u32]>>)
}

struct VkExtent {
	size_x: u32,
	size_y: u32,
	size_z: u32
}

impl VkInstance {
	pub fn new() -> Self {
		let extensions = InstanceExtensions {
			.. InstanceExtensions::none()
		};

		VkInstance {
			instance: Instance::new(None, Version::V1_1, &extensions, None).expect("Failed to create Vulkan instance")
		}
	}

	/// Use the scoped thread pool method of taking a closure and executing it within a scope where vk_instance is always valid
	pub fn with_target<F>(&self, vk_target_scope: F) where F: FnOnce(VkTarget) -> () {
		vk_target_scope(VkTarget::new(self))
	}
}

impl<'a> VkTarget<'a> {
	/// Attempts to find the best Vulkan implementation available and the best QueueFamilies/Queues
	pub fn new(vk_instance: &'a VkInstance) -> Self {
		let device_extensions = DeviceExtensions {
			..DeviceExtensions::none()
		};

		let (physical, queue_family) = VkTarget::select_compute_device(&vk_instance.instance, &device_extensions);

		println!("Using vulkan device: {} (type: {:?})", physical.properties().device_name, physical.properties().device_type);

		let (device, mut queues) = Device::new(
			physical,
			&Features::none(),
			&physical.required_extensions().union(&device_extensions),
			[(queue_family, 0.5)].iter().cloned()
		).expect("Failed to create a device");

		let queue = queues.next().unwrap();

		VkTarget {
			physical,
			queue_family,
			device,
			queue
		}
	}

	// Attempts to find the best Vulkan implementation and QueueFamily
	pub fn select_compute_device(instance: &'a Arc<Instance>, device_extensions: &DeviceExtensions) -> (PhysicalDevice<'a>, QueueFamily<'a>) {
		PhysicalDevice::enumerate(&instance)
			.filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
			.filter_map(|p| {
				// The Vulkan specs guarantee that a compliant implementation must provide at least one queue that supports compute operations
				p.queue_families()
					.find(|&q| q.supports_compute())
					.map(|q| (p, q))
			})
			.min_by_key(|(p, _)| match p.properties().device_type { // Order by device type. Preferably we want to use a discrete gpu
				PhysicalDeviceType::DiscreteGpu => 0,
				PhysicalDeviceType::IntegratedGpu => 1,
				PhysicalDeviceType::VirtualGpu => 2,
				PhysicalDeviceType::Cpu => 3,
				PhysicalDeviceType::Other => 4
			}).expect("No vulkan implementations found")
	}
}

impl<'a> VkComputeOperation<'a> {
	// TODO: This needs to be updated to accept and use arbitrary VkCommands
	pub fn new(vk_target: &'a VkTarget<'a>, storage_bindings: Vec<(VkDataStorage, u32)>, shader_entry_point: (Arc<ShaderModule>, &str), extent: VkExtent) -> Self {
		let pipeline = ComputePipeline::new(
			vk_target.device.clone(),
			shader_entry_point.0.entry_point(shader_entry_point.1).expect(&format!("Entry point \"{}\" not found or multiple instances found", shader_entry_point.1)),
			&(),
			None,
			|_| {}
		).expect("Failed to create compute pipeline");

		let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();

		let descriptor_set = PersistentDescriptorSet::new(
			layout.clone(),
			storage_bindings.iter().map(|(vk_storage, binding)| {
				match vk_storage {
					VkDataStorage::Image(image) => WriteDescriptorSet::image_view(*binding, ImageView::new(image.clone()).expect("Could not create ImageView")),
					VkDataStorage::BufferU8(buffer) => WriteDescriptorSet::buffer(*binding, buffer.clone()),
					_ => todo!("Unhandled VkDataStorage variant") // TODO
				}
			})
		).expect("Failed to create descriptor set");

		let mut builder = AutoCommandBufferBuilder::primary(
			vk_target.device.clone(),
			vk_target.queue_family.clone(),
			CommandBufferUsage::OneTimeSubmit
		).expect("Failed to create AutoCommandBufferBuilder");

		builder
			.bind_pipeline_compute(pipeline.clone())
			.bind_descriptor_sets(PipelineBindPoint::Compute, pipeline.layout().clone(), 0, descriptor_set)
			.dispatch([extent.size_x, extent.size_y, extent.size_z])
			// TODO: Execute arbitrary VkCommands
			.expect("Failed to bind resources and/or add the dispatch command to the AutoCommandBufferBuilder");

		let command_buffer = Arc::new(builder.build().expect("Failed to build command buffer"));

		VkComputeOperation {
			vk_target,
			command_buffer
		}
	}

	// TODO: Add some way of reading data from buffers such as returning read handles
	pub fn execute(&self) {
		let future = sync::now(self.vk_target.device.clone())
			.then_execute(self.vk_target.queue.clone(), self.command_buffer.clone())
			.expect("Failed to send command buffer to GPU for execution")
			.then_signal_fence_and_flush()
			.expect("Failed to instruct GPU to signal fence upon completion and/or flush");

		future.wait(None).expect("Timed out");
	}
}