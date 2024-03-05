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
@group(0) @binding(0) var<storage, read_write> accumulation_buffer: PixelBuffer;
@group(0) @binding(1) var<storage, read> scene: Scene;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var direct_diffuse: texture_2d<f32>;
@group(0) @binding(4) var indirect_diffuse: texture_2d<f32>;
@group(0) @binding(5) var direct_specular: texture_2d<f32>;
@group(0) @binding(6) var indirect_specular: texture_2d<f32>;
@group(0) @binding(7) var sss: texture_2d<f32>;
@group(0) @binding(8) var sky: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Extract
    let index: u32 = u32(in.clip_position.y) * scene.config.size.x + u32(in.clip_position.x);
    let direct_diffuse_color: vec4<f32> = textureSample(direct_diffuse, texture_sampler, in.tex_coords);
    let indirect_diffuse_color: vec4<f32> = textureSample(indirect_diffuse, texture_sampler, in.tex_coords);
    let direct_specular_color: vec4<f32> = textureSample(direct_specular, texture_sampler, in.tex_coords);
    let indirect_specular_color: vec4<f32> = textureSample(indirect_specular, texture_sampler, in.tex_coords);
    let sss_color: vec4<f32> = textureSample(sss, texture_sampler, in.tex_coords);
    let sky_color: vec4<f32> = textureSample(sky, texture_sampler, in.tex_coords);

    // Composite
    let composite = ((direct_diffuse_color + indirect_diffuse_color)) + direct_specular_color + indirect_specular_color + sky_color; // + sss_color
    let accumulated_color: vec4<f32> = composite + accumulation_buffer.data[index];
    accumulation_buffer.data[index] = accumulated_color;

    // Gamma correction
    var out: vec4<f32> = accumulated_color / f32(scene.config.count);
    var gamma: f32 = 2.2; 
    out.r = pow(out.r, 1.0 / gamma);
    out.g = pow(out.g, 1.0 / gamma);
    out.b = pow(out.b, 1.0 / gamma);

    return out;
}
