#version 310 es

in vec3 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

uniform mat4 transform;

void main() {
    gl_Position = transform * vec4(position, 1.0);
    v_tex_coords = tex_coords;
}
