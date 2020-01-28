#version 460 core

in vec2 o_uv;
out vec4 frag_color;

uniform sampler2D image;

void main() {
    vec4 texture_color = texture(image, o_uv);
    
    // discard useless alpha
    if (texture_color.a < 0.1) {
        discard;
    }
    
    frag_color = texture_color;
}