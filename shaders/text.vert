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

// should equal the data.rs
struct Text {
	float vframe; 
	float amount; 
	float centered; 
	vec2 position; 
	float hframe[20]; 
};

layout(set = 0, binding = 3) buffer TextIndexes {
    Text data[];
};

layout(location = 0) out vec2 o_uv;

// how wide a single text is ?
const float TEXT_SIZE = 10; 

void main() {
    Text text = data[gl_InstanceIndex];
	
	for(int i = 0; i < 20; ++i) {
		float hframe = text.hframe[i];
		
		if (hframe == -1) {
			continue;
		}
		
		vec2 position = text.position;
		float text_length = text.amount;
		
		// NOTE(Skytrias): ATLAS DEPENDANT
		int reverse = text.vframe == 8 ? 1 : 0;
		bool centered = text.centered == 1 ? true : false;
		
		// each instance the text gets offset by text width 30
		if (reverse == 0) {
			if (!centered) {
				gl_Position = projection * vec4((rect_pos[gl_VertexIndex] + vec2(i, 0)) * TEXT_SIZE + position, 0.01, 1);
			} else {
				gl_Position = projection * vec4((rect_pos[gl_VertexIndex] + vec2(i - (text_length / 2), 0)) * TEXT_SIZE + position, 0.01, 1);
			}
		} 
		// reverse number drawing
		else {
			gl_Position = projection * vec4((vec2(text_length - 1, 0) + (rect_pos[gl_VertexIndex]) - vec2(i, 0)) * TEXT_SIZE + position, 0.01, 1);
		}
		
		// indexes are gone through each instance, choose right sprite of spritesheet
		vec2 i_uv = vec2(gl_VertexIndex & 1, gl_VertexIndex >> 1);
		o_uv.x = (hframe + i_uv.x) * (1. / 26.);
		o_uv.y = (text.vframe + i_uv.y) * (1. / 11.);
	}
}