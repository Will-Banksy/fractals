use vulkano::instance::{Instance};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::{Pipeline, ComputePipeline, PipelineBindPoint};
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet, WriteDescriptorSet, DescriptorSetsCollection};
use vulkano::shader::ShaderModule;

use std::sync::Arc;

// TODO: This whole file and the structs need an overhaul

pub struct ComputeOperation<T: DescriptorSet> {
	pub shader: Arc<ShaderModule>,
	pub compute_pipeline: Arc<ComputePipeline>,
	pub descriptor_set: Arc<T>
}

impl<T> ComputeOperation<T> where T: DescriptorSet {
	pub fn new(device: Arc<Device>, shader: Arc<ShaderModule>, descriptor_sets: Arc<T>) -> Self {
		let compute_pipeline = ComputePipeline::new(device, shader.entry_point("main").unwrap(), &(), None, |_| {}).unwrap();

		ComputeOperation {
			shader,
			compute_pipeline,
			descriptor_set: descriptor_sets
		}
	}
}

pub struct VulkanComputeInstance<'a> {
	pub physical: PhysicalDevice<'a>,
	pub queue_family: QueueFamily<'a>,
	pub device: Arc<Device>,
	pub queue: Arc<Queue>,
}

impl<'a> VulkanComputeInstance<'a> {
	pub fn new(vk_instance: &'a Arc<Instance>) -> Self {
		let device_extensions = DeviceExtensions {
			..DeviceExtensions::none()
		};

		let (physical, queue_family) = VulkanComputeInstance::select_compute_device(&vk_instance, &device_extensions);

		println!("Using vulkan device: {} (type: {:?})", physical.properties().device_name, physical.properties().device_type);

		let (device, mut queues) = Device::new(
			physical,
			&Features::none(),
			&physical.required_extensions().union(&device_extensions),
			[(queue_family, 0.5)].iter().cloned()
		).expect("Failed to create a device");

		let queue = queues.next().unwrap();

		VulkanComputeInstance {
			physical,
			queue_family,
			device,
			queue
		}
	}

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

	pub fn execute_op<T: 'static>(&self, cop: ComputeOperation<T>, extent: (u32, u32, u32)) where T: DescriptorSet {
		// Command buffer builder - For now, we'll just have it submit once. May do some optimisation later with reusing the command buffer
		let mut builder = AutoCommandBufferBuilder::primary(
			self.device.clone(),
			self.queue_family.clone(),
			CommandBufferUsage::OneTimeSubmit
		).unwrap();

		builder.bind_pipeline_compute(cop.compute_pipeline.clone())
			.bind_descriptor_sets(PipelineBindPoint::Compute, cop.compute_pipeline.layout().clone(), 0, cop.descriptor_set)
			.dispatch([extent.0, extent.1, extent.2])
			.expect("Failed to bind compute pipeline and descriptor sets");

		// Finish building command buffer
		let command_buffer = builder.build().expect("Failed to create command buffer");

		let future = sync::now(self.device.clone())
			.then_execute(self.queue.clone(), command_buffer)
			.unwrap()
			// Instruct the GPU to signal a 'fence' once the command buffer has finished execution. A fence is the GPU telling the CPU that it has reached a certain point
			.then_signal_fence_and_flush()
			.unwrap();

		// Wait until the GPU signals fence
		// Dropping the future would wait anyway this is just more explicit
		future.wait(None).unwrap();
	}
}