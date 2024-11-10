// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#version 310 es

in vec3 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

uniform mat4 transform;

void main() {
    gl_Position = transform * vec4(position, 1.0);
    v_tex_coords = tex_coords;
}
