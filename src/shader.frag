#version 310 es

precision mediump float;

in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D tex;

void main() {
    // Convert XRGB8888 to ABGR8888
    color = vec4(texture(tex, v_tex_coords).zyx, 1.0);
}
