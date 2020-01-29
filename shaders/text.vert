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

// TODO(Skytrias): add offset, centering?, 
// only will ever draw 25
// vec4 layout 
// .x = index to draw character from spritesheet
// .y = x_pos,
// .z = y_pos,
// [0].w vframe
// [1].w = actual size of text
// [2].w = centered by the text length
layout(packed, binding = 3) uniform TextIndexes {
    vec4 indexes[25];
};

out vec2 o_uv;

const float TEXT_SIZE = 10; 

void main() {
    float hframe = indexes[gl_InstanceID].x;
    
    if (hframe == -1) {
        return;
    }
    
    vec2 position = vec2(indexes[gl_InstanceID].y, indexes[gl_InstanceID].z);
    float text_size = indexes[1].w;
	// NOTE(Skytrias): ATLAS DEPENDANT
    int reverse = indexes[0].w == 8 ? 1 : 0;
	bool centered = indexes[2].w == 1 ? true : false;
	
    // each instance the text gets offset by text width 30
    if (reverse == 0) {
        if (!centered) {
			gl_Position = projection * vec4((rect_pos[gl_VertexID] + vec2(gl_InstanceID, 0)) * TEXT_SIZE + position, 0.01, 1);
		} else {
			gl_Position = projection * vec4((rect_pos[gl_VertexID] + vec2(gl_InstanceID - (text_size / 2), 0)) * TEXT_SIZE + position, 0.01, 1);
		}
	} 
    // reverse number drawing
    else {
        gl_Position = projection * vec4((vec2(text_size - 1, 0) + (rect_pos[gl_VertexID]) - vec2(gl_InstanceID, 0)) * TEXT_SIZE + position, 0.01, 1);
    }
    
    // indexes are gone through each instance, choose right sprite of spritesheet
    vec2 i_uv = vec2(gl_VertexID & 1, gl_VertexID >> 1);
    o_uv.x = (hframe + i_uv.x) * (1.0 / 26.);
    o_uv.y = (indexes[0].w + i_uv.y) * (1.0 / 11.);
}