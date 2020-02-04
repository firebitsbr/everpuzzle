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
	vec4 first;
	vec4 second;
} primitive;


layout(set = 0, binding = 3) uniform GridData {
	Block data[72];
};

layout(location = 0) out vec2 o_uv;

const float WIDTH = 6;
const float TILE_SIZE = 32.;

float mod_i(float a,float b) {
    float m = a - floor((a + 0.5) / b) * b;
    return floor(m + 0.5);
}

// .z byte is free

void main() {
    float visible = data[gl_InstanceIndex].first.z;
	
    if (visible == -1) {
        return;
    }
	
	float hframe = data[gl_InstanceIndex].first.x;
	float vframe = data[gl_InstanceIndex].first.y;
	float scale = data[gl_InstanceIndex].first.w;
	
	vec2 offset = vec2(data[gl_InstanceIndex].second.x, data[gl_InstanceIndex].second.y);
	
	vec2 vertice = rect_pos[gl_VertexIndex] - 0.5;
    vec2 position = vec2(mod_i(gl_InstanceIndex, WIDTH), floor((gl_InstanceIndex) / WIDTH)); 
	gl_Position = projection * vec4((vertice * scale + position) * TILE_SIZE + offset + vec2(16, 16), 0.9, 1);
	
    vec2 i_uv = vec2(gl_VertexIndex & 1, gl_VertexIndex >> 1);
    o_uv.x = (hframe + i_uv.x) * (1. / 26.);
    o_uv.y = (vframe + i_uv.y) * (1. / 11.);
}