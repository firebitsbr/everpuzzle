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

struct Block {
	uint hframe;
	uint vframe;
	int visible;
	float scale;
	float x_offset;
	float y_offset;
	float temp1;
	float temp2;
};


layout(set = 0, binding = 3) uniform GridData {
	Block data[72];
};

layout(location = 0) out vec2 o_uv;

const vec2 grid_offset = vec2(100., 50.);
const float WIDTH = 6;
const float TILE_SIZE = 32.;

float mod_i(float a, float b) {
    float m = a - floor((a + 0.5) / b) * b;
    return floor(m + 0.5);
}

// .z byte is free

void main() {
    float visible = data[gl_InstanceIndex].visible;
	
    if (visible == -1) {
        return;
    }
	
	float hframe = data[gl_InstanceIndex].hframe;
	float vframe = data[gl_InstanceIndex].vframe;
	float scale = data[gl_InstanceIndex].scale;
	
	vec2 offset = vec2(data[gl_InstanceIndex].x_offset, data[gl_InstanceIndex].y_offset) + grid_offset;
	
	vec2 vertice = rect_pos[gl_VertexIndex] - 0.5;
    vec2 position = vec2(mod_i(gl_InstanceIndex, WIDTH), floor((gl_InstanceIndex) / WIDTH)); 
	gl_Position = projection * vec4((vertice * scale + position) * TILE_SIZE + offset + vec2(16, 16), 0.9, 1);
	
    vec2 i_uv = vec2(gl_VertexIndex & 1, gl_VertexIndex >> 1);
    o_uv.x = (float(hframe) + i_uv.x) * (1. / 26.);
    o_uv.y = (float(vframe) + i_uv.y) * (1. / 12.);
}