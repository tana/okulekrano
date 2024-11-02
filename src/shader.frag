#version 310 es
#extension GL_OES_EGL_image_external : require

precision mediump float;

in vec2 v_tex_coords;

out vec4 color;

uniform samplerExternalOES tex;

void main() {
    color = texture2D(tex, v_tex_coords);
}
