struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Currently texture coordinates are just mapped to color
    // TOOD: texture mapping
    return vec4<f32>(in.tex_coords, 0.0, 1.0);
}