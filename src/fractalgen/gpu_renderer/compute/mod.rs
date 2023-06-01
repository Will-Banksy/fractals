pub mod shaders;
pub mod vulkan_compute;

use crate::fractalgen::FractalType;
use crate::fractalgen::PlaneTransform;
use crate::fractalgen::gpu_renderer::compute::vulkan_compute::VkComputeOperation;
use crate::fractalgen::gpu_renderer::compute::vulkan_compute::VkDataStorage;
use crate::fractalgen::gpu_renderer::compute::vulkan_compute::VkExtent;
use crate::fractalgen::gpu_renderer::compute::vulkan_compute::VkInstance;

use image::RgbaImage;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::image::ImageDimensions;
use vulkano::image::StorageImage;
use vulkano::format::Format;

#[cfg(test)]
mod test_shader {
	vulkano_shaders::shader!{
		ty: "compute",
		path: "shaders/test_shader.comp"
	}
}

// TODO: Test VkCommands
#[test]
#[cfg(test)]
fn test_vulkan_compute() {
	use crate::fractalgen::gpu_renderer::compute::vulkan_compute::{VkComputeOperation, VkDataStorage, VkExtent, VkInstance};
	use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

	let vk_instance = VkInstance::new();

	vk_instance.with_target(|vk_target| {
		let data = vec![
			VkDataStorage::BufferU32(
				CpuAccessibleBuffer::from_iter(vk_target.device.clone(), BufferUsage::all(), false,
					(0..16000).map(|_| 0)).expect("Failed to create buffer")
			),
			VkDataStorage::BufferU32(
				CpuAccessibleBuffer::from_iter(vk_target.device.clone(), BufferUsage::all(), false,
					(0..16000).map(|_| 0)).expect("Failed to create buffer")
			)
		];
		let data_bindings = vec![vec![0]];
		let shader = test_shader::load(vk_target.device.clone()).expect("Failed to create shader");
		let mut op = VkComputeOperation::new(&vk_target, &data, &data_bindings, (shader, "main"), VkExtent::new(16000, 1, 1));
		op.dispatch().build().execute().unwrap();

		// Check
		if let VkDataStorage::BufferU32(buffer) = &data[0] {
			let content = buffer.read().expect("Failed to read buffer");
			for (i, val) in content.iter().enumerate() {
				assert_eq!(i as u32, *val);
			}
			println!("Test was successful!")
		}
	});
}

// TODO: Use fractal_type and pass transform and max_iterations to shader in a buffer (need new type of buffer in VkDataStorage. *Sigh* ideally you should be able to use any type... Need to think of how to do this)
pub fn generate_fractal_image<'a>(fractal_type: FractalType, dimensions: (u32, u32), transform: &PlaneTransform<f64>, max_iterations: Option<u32>) -> RgbaImage {
	let (width, height) = dimensions;

	let vk_instance = VkInstance::new();

	vk_instance.with_target(|vk_target| {
		let data = vec![
			VkDataStorage::Image(
				StorageImage::new(vk_target.device.clone(), ImageDimensions::Dim2d { width, height, array_layers: 1 },
					Format::R8G8B8A8_UNORM, Some(vk_target.queue_family)).expect("Failed to create image")
			),
			VkDataStorage::BufferU8(
				CpuAccessibleBuffer::from_iter(vk_target.device.clone(), BufferUsage::all(), false,
					(0..(width * height * 4)).map(|_| 0)).expect("Failed to create buffer")
			)
		];
		let data_bindings = vec![vec![0]];
		let shader = shaders::mandelbrot::load(vk_target.device.clone()).expect("Failed to create shader");
		let mut op = VkComputeOperation::new(&vk_target, &data, &data_bindings, (shader, "main"), VkExtent::new(width, height, 1));
		op.dispatch().add_commands(vec![
			|builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>| {
				let image = data[0].image().unwrap();
				let buffer = data[1].buffer_u8().unwrap();
				builder.copy_image_to_buffer(image.clone(), buffer.clone())
					.expect("Failed to submit copy command to builder");
			}
		]).build().execute().unwrap();

		let working_buffer_content = data[1].buffer_u8().unwrap().read().unwrap()[..].to_vec();

		RgbaImage::from_raw(width, height, working_buffer_content).unwrap()
	})

	// todo!();

	// // Create vulkan compute instance
	// let vk_comp = VulkanComputeInstance::new(&vk_instance);

	// let device = vk_comp.device.clone();
	// let shader = shaders::mandelbrot::load(device.clone()).expect("Failed to create shader");
	// let compute_pipeline = ComputePipeline::new(device.clone(), shader.entry_point("main").unwrap(), &(), None, |_| {}).unwrap();

	// let working_img = StorageImage::new(
	// 	device.clone(),
	// 	vulkano::image::ImageDimensions::Dim2d { width, height, array_layers: 1 },
	// 	Format::R8G8B8A8_UNORM,
	// 	Some(vk_comp.queue_family)
	// ).expect("Failed to create image");
	// let working_img_view = ImageView::new(working_img.clone()).unwrap();

	// let result_buffer = CpuAccessibleBuffer::from_iter(
	// 	device.clone(),
	// 	BufferUsage::all(),
	// 	false,
	// 	(0..(width * height * 4)).map(|_| 0)
	// ).expect("Failed to create buffer");

	// let layout = compute_pipeline.layout().descriptor_set_layouts().get(0).unwrap();

	// let descriptor_set = PersistentDescriptorSet::new(layout.clone(), [WriteDescriptorSet::image_view(0, working_img_view.clone())]).unwrap();

	// // Command buffer builder - For now, we'll just have it submit once. May do some optimisation later with reusing the command buffer
	// let mut builder = AutoCommandBufferBuilder::primary(
	// 	device.clone(),
	// 	vk_comp.queue_family.clone(),
	// 	CommandBufferUsage::OneTimeSubmit
	// ).unwrap();

	// builder
	// 	.bind_pipeline_compute(compute_pipeline.clone())
	// 	.bind_descriptor_sets(PipelineBindPoint::Compute, compute_pipeline.layout().clone(), 0, descriptor_set)
	// 	.dispatch([width, height, 1])
	// 	.unwrap()
	// 	.copy_image_to_buffer(working_img.clone(), result_buffer.clone())
	// 	.unwrap();

	// let command_buffer = builder.build().unwrap();

	// let future = vulkano::sync::now(device.clone())
	// 	.then_execute(vk_comp.queue.clone(), command_buffer)
	// 	.unwrap()
	// 	// Instruct the GPU to signal a 'fence' once the command buffer has finished execution. A fence is the GPU telling the CPU that it has reached a certain point
	// 	.then_signal_fence_and_flush()
	// 	.unwrap();

	// // Wait until the GPU signals fence
	// // Dropping the future would wait anyway this is just more explicit
	// future.wait(None).unwrap();

	// // let compute_op = ComputeOperation {
	// // 	shader,
	// // 	compute_pipeline,
	// // 	descriptor_set
	// // };

	// // vk_comp.execute_op(compute_op, (width, height, 1)); // This doesn't allow to specify what commands to include in the command buffer, such as copy commands

	// let working_buffer_content = result_buffer.read().unwrap();

	// RgbaImage::from_raw(width, height, working_buffer_content[..].to_vec()).unwrap()
}