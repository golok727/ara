struct Globals {
    proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;


struct VertexIn {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
};

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(1) uv: vec2f,
    @location(0) color: vec4f,
};

@vertex fn vs(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    let proj = transpose(globals.proj);
    out.position = proj * vec4f(in.position, 1.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

// TODO add polychrome and monochrome
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var tex_sampler: sampler;

@fragment fn fs_poly(in: VertexOut)-> @location(0) vec4f {
    let tex_color = textureSample(tex, tex_sampler, in.uv);
    return in.color * tex_color;
}

@fragment
fn fs_mono(in: VertexOut) -> @location(0) vec4f {
    let tex_color = textureSample(tex, tex_sampler, in.uv);
    return in.color * tex_color.r;
}



