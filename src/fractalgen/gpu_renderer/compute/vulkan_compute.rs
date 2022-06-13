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

#[derive(Clone)]
pub struct VkInstance {
	pub instance: Arc<Instance>
}

pub struct VkTarget<'a> {
	pub physical: PhysicalDevice<'a>,
	pub queue_family: QueueFamily<'a>,
	pub device: Arc<Device>,
	pub queue: Arc<Queue>,
}

pub struct VkComputeOperation<'a> {
	pub vk_target: &'a VkTarget<'a>,
	pub command_buffer_builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
	pub command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>,
	pub data: &'a Vec<VkDataStorage>,
	pub data_bindings: &'a Vec<Vec<u32>>,
	pub extent: VkExtent
}

#[derive(Clone)]
pub enum VkDataStorage {
	Image(Arc<StorageImage>),
	BufferU8(Arc<CpuAccessibleBuffer<[u8]>>),
	BufferU32(Arc<CpuAccessibleBuffer<[u32]>>)
}

#[derive(Clone, Copy)]
pub struct VkExtent {
	pub size_x: u32,
	pub size_y: u32,
	pub size_z: u32
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
	pub fn with_target<F, R>(&self, vk_target_scope: F) -> R where F: FnOnce(VkTarget) -> R {
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
	/// If any dimension of `extent` is 0, the shader will not run
	pub fn new(vk_target: &'a VkTarget<'a>, data: &'a Vec<VkDataStorage>, data_bindings: &'a Vec<Vec<u32>>, shader_entry_point: (Arc<ShaderModule>, &str), extent: VkExtent) -> Self {
		let pipeline = ComputePipeline::new(
			vk_target.device.clone(),
			shader_entry_point.0.entry_point(shader_entry_point.1).expect(&format!("Entry point \"{}\" not found or multiple instances found", shader_entry_point.1)),
			&(),
			None,
			|_| {}
		).expect("Failed to create compute pipeline");

		let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();

		let descriptor_sets: Vec<Arc<PersistentDescriptorSet>> = data_bindings.iter().map(|set_bindings| {
			PersistentDescriptorSet::new(layout.clone(), set_bindings.iter().enumerate().map(|(binding, i)| {
				match &data[*i as usize] {
					VkDataStorage::Image(image) => WriteDescriptorSet::image_view(binding as u32, ImageView::new(image.clone()).expect("Could not create ImageView")),
					VkDataStorage::BufferU8(buffer) => WriteDescriptorSet::buffer(binding as u32, buffer.clone()),
					VkDataStorage::BufferU32(buffer) => WriteDescriptorSet::buffer(binding as u32, buffer.clone()),
				}
			})).expect("Failed to create descriptor set")
		}).collect();

		let mut builder = AutoCommandBufferBuilder::primary(
			vk_target.device.clone(),
			vk_target.queue_family.clone(),
			CommandBufferUsage::OneTimeSubmit
		).expect("Failed to create AutoCommandBufferBuilder");

		builder
			.bind_pipeline_compute(pipeline.clone())
			.bind_descriptor_sets(PipelineBindPoint::Compute, pipeline.layout().clone(), 0, descriptor_sets);

		VkComputeOperation {
			vk_target,
			command_buffer_builder: Some(builder),
			command_buffer: None,
			data,
			data_bindings,
			extent
		}
	}

	/// Allows adding VkCommands before the dispatch call
	pub fn add_commands<VkCommand>(&mut self, vk_commands: Vec<VkCommand>) -> &mut Self where VkCommand: FnOnce(&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> () {
		if let Some(builder) = &mut self.command_buffer_builder {
			// Execute supplied VkCommands
			for command in vk_commands {
				command(builder);
			}
		};
		self
	}

	/// Adds the dispatch call to the command buffer builder
	pub fn dispatch(&mut self) -> &mut Self {
		if let Some(builder) = &mut self.command_buffer_builder {
			builder
				.dispatch([self.extent.size_x, self.extent.size_y, self.extent.size_z])
				.expect("Failed to add the dispatch command to the AutoCommandBufferBuilder");
		}
		self
	}

	/// Builds the command buffer
	pub fn build(&mut self) -> &mut Self {
		if let Some(builder) = self.command_buffer_builder.take() {
			self.command_buffer = Some(
				Arc::new(builder.build().expect("Failed to build command buffer"))
			);
		};
		self
	}

	/// Submits the command buffer to the GPU to be executed
	pub fn execute(&self) -> Result<(), &str> {
		match &self.command_buffer {
			Some(command_buffer) => {
				let future = sync::now(self.vk_target.device.clone())
					.then_execute(self.vk_target.queue.clone(), command_buffer.clone())
					.expect("Failed to send command buffer to GPU for execution")
					.then_signal_fence_and_flush()
					.expect("Failed to instruct GPU to signal fence upon completion and/or flush");

				future.wait(None).expect("Timed out");

				Ok(())
			},
			None => Err("Command buffer was not built")
		}
	}
}

impl VkDataStorage {
	pub fn image(&self) -> Option<Arc<StorageImage>> {
		match self {
			VkDataStorage::Image(res) => Some(res.clone()),
			_ => None
		}
	}

	pub fn buffer_u8(&self) -> Option<Arc<CpuAccessibleBuffer<[u8]>>> {
		match self {
			VkDataStorage::BufferU8(res) => Some(res.clone()),
			_ => None
		}
	}

	pub fn buffer_u32(&self) -> Option<Arc<CpuAccessibleBuffer<[u32]>>> {
		match self {
			VkDataStorage::BufferU32(res) => Some(res.clone()),
			_ => None
		}
	}
}

impl VkExtent {
	pub fn new(size_x: u32, size_y: u32, size_z: u32) -> Self {
		VkExtent { size_x, size_y, size_z }
	}
}