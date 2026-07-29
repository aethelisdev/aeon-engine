// Shadow Depth Pass Shader
// Fragment output yok - sadece depth buffer doldurulur.

struct CascadeUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> cascade: CascadeUniform;

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,
}

@vertex
fn vs_shadow(
    @location(0) position: vec3<f32>,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let world_pos = model_matrix * vec4<f32>(position, 1.0);
    return cascade.matrix * world_pos;
}
