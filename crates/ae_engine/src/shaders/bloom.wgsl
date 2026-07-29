// Bloom & Post-Processing Shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0)
    );
    var uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );

    var out: VertexOutput;
    out.position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

// Extract bright pixels (Luminance threshold)
@fragment
fn fs_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (brightness > 0.8) {
        return color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

// Simple Gaussian Blur (Horizontal/Vertical)
@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    var weight = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    let texel_size = 1.0 / tex_size;
    
    var result = textureSample(t_diffuse, s_diffuse, in.uv) * weight[0];
    
    for(var i: i32 = 1; i < 5; i++) {
        result += textureSample(t_diffuse, s_diffuse, in.uv + vec2<f32>(texel_size.x * f32(i), 0.0)) * weight[i];
        result += textureSample(t_diffuse, s_diffuse, in.uv - vec2<f32>(texel_size.x * f32(i), 0.0)) * weight[i];
        result += textureSample(t_diffuse, s_diffuse, in.uv + vec2<f32>(0.0, texel_size.y * f32(i))) * weight[i];
        result += textureSample(t_diffuse, s_diffuse, in.uv - vec2<f32>(0.0, texel_size.y * f32(i))) * weight[i];
    }
    
    return result;
}

@group(1) @binding(0) var t_bloom: texture_2d<f32>;
@group(1) @binding(1) var s_bloom: sampler;

struct BloomParams {
    intensity: f32,
};
@group(2) @binding(0) var<uniform> bloom_params: BloomParams;

// Composition Pass (Scene + Bloom) — now uses bloom_params.intensity
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let bloom_color = textureSample(t_bloom, s_bloom, in.uv);
    
    // Additive blending scaled by intensity + simple tone mapping
    let combined = scene_color.rgb + bloom_color.rgb * bloom_params.intensity;
    let result = 1.0 - exp(-combined * 1.0); // Exposure tone mapping
    
    return vec4<f32>(result, 1.0);
}
