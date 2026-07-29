struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(6) normal: vec3<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,
}

struct Light {
    direction: vec3<f32>, // A normalized vector pointing TOWARDS the sun source
    color: vec3<f32>,
    ambient_color: vec3<f32>,
    fog_params: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> light: Light;

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,   // Must declare to keep correct memory layout
    proj_inv: mat4x4<f32>,   // Must declare to keep correct memory layout
    camera_pos: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// --- Shadow Map ---
struct LightSpaceUniform {
    matrices: array<mat4x4<f32>, 4>,
    cascade_splits: vec4<f32>,
    shadow_bias: f32,
    pcf_radius: i32,
    shadow_enabled: u32,
    _pad: u32,
}

@group(2) @binding(0) var shadow_map: texture_depth_2d_array;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;
@group(2) @binding(2) var<uniform> light_space: LightSpaceUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) frag_pos_light_space: vec4<f32>, // unused explicitly now
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    let world_p = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_p;
    out.color = vec4<f32>(model.color, 1.0) * instance.color;
    out.world_pos = world_p.xyz;
    out.world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    return out;
}

// PCF shadow sampling
fn sample_shadow_pcf(world_pos: vec3<f32>) -> f32 {
    if light_space.shadow_enabled == 0u {
        return 1.0; 
    }

    let dist = distance(world_pos, camera.camera_pos.xyz);
    var cascade_idx: i32 = 3;
    if dist < light_space.cascade_splits[0] {
        cascade_idx = 0;
    } else if dist < light_space.cascade_splits[1] {
        cascade_idx = 1;
    } else if dist < light_space.cascade_splits[2] {
        cascade_idx = 2;
    }

    let frag_pos_light = light_space.matrices[cascade_idx] * vec4<f32>(world_pos, 1.0);
    var proj = frag_pos_light.xyz / frag_pos_light.w;
    let uv = vec2<f32>(proj.x * 0.5 + 0.5, -proj.y * 0.5 + 0.5);

    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return 1.0;
    }

    let current_depth = proj.z - light_space.shadow_bias;
    let radius = light_space.pcf_radius;

    if radius == 0 {
        return textureSampleCompare(shadow_map, shadow_sampler, uv, cascade_idx, current_depth);
    }

    var shadow_sum = 0.0;
    var sample_count = 0.0;
    let texel_size = 1.0 / 2048.0;

    // Compile-time upper bound for the PCF kernel. FXC (DX12) requires
    // loop bounds to be known at compile time for unrolling. The actual
    // runtime radius is still respected via the inner continue guard.
    const MAX_PCF: i32 = 4;

    for (var x = -MAX_PCF; x <= MAX_PCF; x = x + 1) {
        for (var y = -MAX_PCF; y <= MAX_PCF; y = y + 1) {
            if (abs(x) > radius || abs(y) > radius) { continue; }
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow_sum += textureSampleCompare(shadow_map, shadow_sampler, uv + offset, cascade_idx, current_depth);
            sample_count += 1.0;
        }
    }
    return shadow_sum / sample_count;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Scene Ambient Lighting (Derived from Sky Mood)
    let ambient_color = light.ambient_color;

    // 2. Direct Sun Illumination
    let normal = normalize(in.world_normal);
    
    // light.direction is pointing TOWARDS the light source
    let light_dir = normalize(light.direction);
    
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse_color = light.color * diff;

    // 3. Shadow factor [0.0 = full shadow, 1.0 = lit]
    let shadow = sample_shadow_pcf(in.world_pos);

    // Combine lighting
    var color_out = (ambient_color + diffuse_color * shadow) * in.color.rgb;

    // 4. Atmospheric Depth Fog
    let fog_distance = light.fog_params.w;
    if (fog_distance > 0.0) {
        let dist = distance(in.world_pos, camera.camera_pos.xyz);
        // Start fog at 50 units, fully occlude at fog_distance
        let fog_factor = clamp((dist - 50.0) / (fog_distance - 50.0), 0.0, 1.0);
        let fog_color = light.fog_params.xyz;
        color_out = mix(color_out, fog_color, fog_factor);
    }

    return vec4<f32>(color_out, in.color.a);
}
