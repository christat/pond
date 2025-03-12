#version 460
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 out_color;
layout (location = 1) out vec2 out_uv;

// POD shared with CPU; note UV packing for alignment
struct Vertex
{
    vec4 position_uv_x;
	vec4 normal_uv_y;
	vec4 color;
}; 

// Direct buffer access declaration, with alignment
layout (buffer_reference, std430) readonly buffer VertexBuffer
{ 
	Vertex vertices[];
};

layout (push_constant) uniform constants
{	
	mat4 render_matrix;
	VertexBuffer vertex_buffer;
} PushConstants;

void main() 
{	
	// load vertex data from device address
	Vertex vertex = PushConstants.vertex_buffer.vertices[gl_VertexIndex];
	// output vertex data
	gl_Position = PushConstants.render_matrix * vec4(vertex.position_uv_x.xyz, 1.0f);
	out_color = vertex.color.xyz;
	out_uv.x = vertex.position_uv_x.w;
	out_uv.y = vertex.normal_uv_y.w;
}
