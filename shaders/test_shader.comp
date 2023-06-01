#version 450

// Vulkan will ask the GPU to spawn a number of work groups
// This line declares the size/ranges that a work group should cover (Should always aim for work group size to be at least 32/64)
// Data can be up to 3-dimensional - Use x for 1D data, x and y for 2D, and x, y and z for 3D data (set rest to 1)
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

// Binding 0 in descriptor set 0
layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
	uint idx = gl_GlobalInvocationID.x;
	buf.data[idx] = idx;
}