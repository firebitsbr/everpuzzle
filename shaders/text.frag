#version 460 core

layout(location = 0) in vec2 o_uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 1) uniform texture2D texture_color;
layout(set = 0, binding = 2) uniform sampler sampler_color;

void main() {
    // TODO(Skytrias): discard necesary with wgpu?
	vec4 texture_color = texture(sampler2D(texture_color, sampler_color), o_uv);
	
    // discard useless alpha
    if (texture_color.a < 0.1) {
        discard;
    }
	
    frag_color = texture_color;
}