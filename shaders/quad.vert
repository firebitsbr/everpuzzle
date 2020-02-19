#version 450

const vec2 rect_pos[4] = vec2[4](
								 vec2(0, 0),
								 vec2(1, 0),
								 vec2(0, 1),
								 vec2(1, 1)
								 );

layout(set = 0, binding = 0) uniform Globals {
    mat4 projection;
};

// wgpu doesnt support VertexFormat Mat4, so i piece them together
layout(location = 0) in vec4 first;
layout(location = 1) in vec4 second;
layout(location = 2) in vec4 third;
layout(location = 3) in vec4 fourth;
layout(location = 4) in vec2 tiles;
layout(location = 5) in float hframe;
layout(location = 6) in float vframe;
layout(location = 7) in float depth;

layout(location = 0) out vec2 o_uv;

void main() {
	vec2 v_pos = rect_pos[gl_VertexIndex];
	
	mat4 i_model = mat4(
							first,
							second,
							third,
							fourth
							);
	
	gl_Position = projection * i_model * vec4(v_pos, depth, 1.);
	
	vec2 i_uv = vec2(gl_VertexIndex & 1, gl_VertexIndex >> 1) * tiles;
	o_uv.x = (hframe + i_uv.x) * (1. / 8.);
    o_uv.y = (vframe + i_uv.y) * (1. / 10.);
}