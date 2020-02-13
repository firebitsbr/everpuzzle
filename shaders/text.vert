#version 450

const vec2 rect_pos[4] = vec2[4](
								 vec2(0, 0),
								 vec2(1, 0),
								 vec2(0, 1),
								 vec2(1, 1)
								 );

layout(set = 0, binding = 0) uniform Global {
    mat4 projection;
};

// only will ever draw 25
// vec4 layout 
// .x = index to draw character from spritesheet
// .y = x_pos,
// .z = y_pos,
// [0].w vframe
// [1].w = actual size of text
// [2].w = centered by the text length
layout(set = 0, binding = 3) buffer TextIndexes {
    vec4 indexes[20];
};

layout(location = 0) out vec2 o_uv;
layout(location = 1) out float o_not_visible;

void main() {
	float hframe = indexes[gl_InstanceIndex].x;
	
	o_not_visible = float(hframe == -1.);
	
	vec2 position = vec2(indexes[gl_InstanceIndex].y, indexes[gl_InstanceIndex].z);
	float text_length = indexes[1].w;
	int reverse = int(indexes[0].w == 8);
	// NOTE(Skytrias): ATLAS DEPENDANT
	bool centered = indexes[2].w == 1 ? true : false;
	vec2 text_size = vec2(indexes[3].w, indexes[4].w);
	
	// each instance the text gets offset by text width 30
	if (reverse == 0) {
		if (!centered) {
			gl_Position = projection * vec4((rect_pos[gl_VertexIndex] + vec2(gl_InstanceIndex, 0)) * text_size + position, 0.01, 1);
		} else {
			gl_Position = projection * vec4((rect_pos[gl_VertexIndex] + vec2(gl_InstanceIndex - (text_length / 2), 0)) * text_size + position, 0.01, 1);
		}
	} 
	// reverse number drawing
	else {
		gl_Position = projection * vec4((vec2(text_length - 1, 0) + (rect_pos[gl_VertexIndex]) - vec2(gl_InstanceIndex, 0)) * text_size + position, 0.01, 1);
	}
	
	// indexes are gone through each instance, choose right sprite of spritesheet
	vec2 i_uv = vec2(gl_VertexIndex & 1, gl_VertexIndex >> 1);
	o_uv.x = (hframe + i_uv.x) * (1. / 26.);
	o_uv.y = (indexes[0].w  + i_uv.y) * (1. / 12.);
}