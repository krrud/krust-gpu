struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Define the vertices of the full-screen quad as a constant
const vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0)
);

@vertex
fn vs_main(
    @builtin(vertex_index) VertexIndex : u32
) -> VertexOutput {
    var out: VertexOutput;
    var vertex: vec2<f32>;

    let index = VertexIndex % 3u;
    if (index == 0u) { vertex = vec2<f32>(-1.0, -1.0); }
    else if (index == 1u) { vertex = vec2<f32>(3.0, -1.0); }
    else if (index == 2u) { vertex = vec2<f32>(-1.0, 3.0); }

    out.tex_coords = vec2<f32>((vertex.x + 1.0) / 2.0, 1.0 - (vertex.y + 1.0) / 2.0);
    out.clip_position = vec4<f32>(vertex, 0.0, 1.0);
    return out;
}

// Fragment shader
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_trace: sampler;
@group(0) @binding(2) var t_trace: texture_2d<f32>;
@group(0) @binding(3) var<storage, read_write> accumulation_buffer: PixelBuffer;
@group(0) @binding(4) var<storage, read> scene: Scene;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let index: u32 = u32(in.clip_position.y) * scene.config.size.x + u32(in.clip_position.x);
    let trace_color: vec4<f32> = textureSample(t_trace, s_trace, in.tex_coords);
    let accumulated_color: vec4<f32> = trace_color + accumulation_buffer.data[index];
    var composite: vec4<f32> = accumulated_color / f32(scene.config.count);

    accumulation_buffer.data[index] = accumulated_color;

    // Gamma correction
    var gamma: f32 = 1.0; 
    composite.r = pow(composite.r, 1.0 / gamma);
    composite.g = pow(composite.g, 1.0 / gamma);
    composite.b = pow(composite.b, 1.0 / gamma);

    return composite;
}
