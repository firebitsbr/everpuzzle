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


struct Block {
	vec4 first;
	vec4 second;
} primitive;


layout(packed, binding = 2) uniform GridData {
	Block data[20];
};

out vec2 o_uv;

const float WIDTH = 6;

float mod_i(float a,float b) {
    float m = a - floor((a + 0.5) / b) * b;
    return floor(m + 0.5);
}

// .z byte is free

const float TILE_SIZE = 32.;

void main() {
    float visible = data[gl_InstanceID].first.z;
	
    if (visible == -1) {
        return;
    }
	
	float hframe = data[gl_InstanceID].first.x;
	float vframe = data[gl_InstanceID].first.y;
	float scale = data[gl_InstanceID].first.w;
	
	vec2 offset = vec2(data[gl_InstanceID].second.x, data[gl_InstanceID].second.y);
	
	vec2 vertice = rect_pos[gl_VertexID] - 0.5;
    vec2 position = vec2(mod_i(gl_InstanceID, WIDTH), floor((gl_InstanceID) / WIDTH)); 
	gl_Position = projection * vec4((vertice * scale + position) * TILE_SIZE + offset + vec2(16, 16), 0.2, 1);
    
    vec2 i_uv = vec2(gl_VertexID & 1, gl_VertexID >> 1);
    o_uv.x = (hframe + i_uv.x) * (1. / 26.);
    o_uv.y = (vframe + i_uv.y) * (1. / 10.);
}