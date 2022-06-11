#version 450

// Vulkan will ask the GPU to spawn a number of work groups
// This line declares the size/ranges that a work group should cover (Should always aim for work group size to be at least 32/64)
// Data can be up to 3-dimensional - Use x for 1D data, x and y for 2D, and x, y and z for 3D data (set rest to 1)
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

// Declare a descriptor - Descriptor 0 in Descriptor Set 0
// Declare a writeonly 2D image
layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
	uint idx = gl_GlobalInvocationID.x;
	vec4 pixdat = vec4(gl_GlobalInvocationID.x, 0.0, 0.0, 1.0);
	imageStore(img, ivec2(gl_GlobalInvocationID.xy), pixdat);
}