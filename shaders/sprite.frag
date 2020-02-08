#version 450

layout(location = 0) in vec2 o_uv;
layout(location = 1) in vec4 o_color;
layout(location = 2) in float o_visible;
layout(location = 0) out vec4 frag_color;

// NOTE(Skytrias): use new set?
layout(set = 0, binding = 1) uniform texture2D texture_color;
layout(set = 0, binding = 2) uniform sampler sampler_color;

void main() {
	if (o_visible != 1.) {
		discard;
	}
	
	vec4 texture_color = texture(sampler2D(texture_color, sampler_color), o_uv);
	
    // discard useless alpha
    if (texture_color.a < 0.1) {
        discard;
    }
	
	frag_color = texture_color * o_color;
}