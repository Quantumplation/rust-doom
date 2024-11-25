// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    var x: f32;
    var y: f32;
    switch in_vertex_index {
        case 0u, 3u: {
            x = -1.0;
            y = 1.0;
        }
        case 1u: {
            x = -1.0;
            y = -1.0;
        }
        case 2u, 4u: {
            x = 1.0;
            y = -1.0;
        }
        default: {
            x = 1.0;
            y = 1.0;
        }
    }
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>((x + 1.0) / 2.0, (1.0 - y) / 2.0);
    return out;
}

// Fragment shader
@group(0) @binding(0)
var u_texture: texture_2d<f32>;
@group(0) @binding(1)
var u_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, in.tex_coords);
}
