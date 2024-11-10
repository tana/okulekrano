// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#version 310 es

precision mediump float;

in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D tex;

void main() {
    // Convert XRGB8888 to ABGR8888
    color = vec4(texture(tex, v_tex_coords).zyx, 1.0);
}
