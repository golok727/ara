struct Globals {
    viewport_size: vec2f,
    _pad: vec2f, // webgl requirement
};

fn to_device_coordinate(point: vec2f) -> vec2f {
    return (point / globals.viewport_size) * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0);
}

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
    out.position = vec4f(to_device_coordinate(in.position), 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment fn fs_main(in: VertexOut)-> @location(0) vec4f {
    return in.color;
}
