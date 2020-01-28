#version 460 core

const vec2 rect_pos[4] = vec2[4](
								 vec2(0, 0),
								 vec2(1, 0),
								 vec2(0, 1),
								 vec2(1, 1)
								 );

layout(std140, binding = 0) uniform Global {
    mat4 projection;
};

struct Primitive {
	vec4 color;
	vec4 pos_and_dim;
	vec4 offset_and_scale;
	vec4 additional;
	vec4 additional2;
} primitive;

// TODO(Skytrias): add z 
const int PRIMITIVE_MAX = 100;
layout(packed, binding = 1) uniform PrimitiveData {
    Primitive data[PRIMITIVE_MAX];
};

out vec2 o_uv;
out vec4 o_color;

const float TILE_SIZE = 32.;

void main() {
	// check if primitive is even visible
	if (data[gl_InstanceID].additional.w == 0.) {
		return;
	}
	
    vec2 position = data[gl_InstanceID].pos_and_dim.xy;
	vec2 dimensions = data[gl_InstanceID].pos_and_dim.zw;
	vec2 offset = data[gl_InstanceID].offset_and_scale.xy;
	vec2 scale = data[gl_InstanceID].offset_and_scale.zw;
	float rotation = data[gl_InstanceID].additional.x;
	float hframe = data[gl_InstanceID].additional.y;
	float vframe = data[gl_InstanceID].additional.z;
	float depth = data[gl_InstanceID].additional2.x;
	//float centered = data[gl_InstanceID].additional2.x;
	
	if (rotation == 0.) {
		gl_Position = projection * vec4(((rect_pos[gl_VertexID] * dimensions) * scale) + position, depth, 1.);
	} else {
		// rotate around center, have to account ratio in between dimensions and texture
		vec2 vertice = rect_pos[gl_VertexID];
		vec2 ratio = (dimensions / vec2(TILE_SIZE, TILE_SIZE));
		vertice *= ratio;
		vec2 center = (offset * ratio) / dimensions;
		
		vec2 rotated_pos = vec2(
								cos(rotation) * (vertice.x - center.x) - sin(rotation) * (vertice.y - center.y) + center.x,
								sin(rotation) * (vertice.x - center.x) + cos(rotation) * (vertice.y - center.y) + center.y
								);
		
		gl_Position = projection * vec4(((rotated_pos * vec2(TILE_SIZE, TILE_SIZE)) * scale ) + position, depth, 1.);
	}
	
    vec2 i_uv = vec2(gl_VertexID & 1, gl_VertexID >> 1) * (dimensions / vec2(TILE_SIZE, TILE_SIZE));
	// make uv be the same scale as the dimensions are i.e. 2 to 1
	o_uv.x = (hframe + i_uv.x) * (1. / 26.);
    o_uv.y = (vframe + i_uv.y) * (1. / 10.);
	o_color = data[gl_InstanceID].color;
}