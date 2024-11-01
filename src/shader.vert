#version 310 es

in vec3 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
    gl_Position = vec4(position, 1.0);
    v_tex_coords = tex_coords;
}