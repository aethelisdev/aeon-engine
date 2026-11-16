// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

// Iris UI GPU SDF Quad Fragment and Vertex Shader
// Renders rounded boxes, borders, and shadows with sub-pixel antialiasing in a single pass.

struct Uniforms {
    screen_size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_globals: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>, // Unit quad [0.0 .. 1.0]
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,              // [x, y, width, height]
    @location(2) color: vec4<f32>,             // Background RGBA
    @location(3) border_color: vec4<f32>,      // Border RGBA
    @location(4) border_width: vec4<f32>,      // [top, right, bottom, left]
    @location(5) corner_radii: vec4<f32>,      // [top_left, top_right, bottom_right, bottom_left]
    @location(6) shadow_color: vec4<f32>,      // Shadow RGBA
    @location(7) shadow_params: vec4<f32>,     // [offset_x, offset_y, blur, spread]
    @location(8) clip_rect: vec4<f32>,         // [min_x, min_y, max_x, max_y]
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) rect_size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) border_width: vec4<f32>,
    @location(5) corner_radii: vec4<f32>,
    @location(6) shadow_color: vec4<f32>,
    @location(7) shadow_params: vec4<f32>,
    @location(8) screen_pos: vec2<f32>,
    @location(9) clip_bounds: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let shadow_blur = instance.shadow_params.z;
    let shadow_spread = instance.shadow_params.w;
    let pad = max(shadow_blur * 2.0 + max(shadow_spread, 0.0), 2.0);

    // Expand vertex bounds to cover soft shadow margin
    let quad_min = instance.rect.xy - vec2<f32>(pad);
    let quad_size = instance.rect.zw + vec2<f32>(pad * 2.0);
    let pixel_pos = quad_min + vertex.position * quad_size;

    // Convert pixel position to Normalized Device Coordinates [-1.0, 1.0]
    let ndc_x = (pixel_pos.x / u_globals.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / u_globals.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local_pos = pixel_pos - instance.rect.xy;
    out.rect_size = instance.rect.zw;
    out.color = instance.color;
    out.border_color = instance.border_color;
    out.border_width = instance.border_width;
    out.corner_radii = instance.corner_radii;
    out.shadow_color = instance.shadow_color;
    out.shadow_params = instance.shadow_params;
    out.screen_pos = pixel_pos;
    out.clip_bounds = instance.clip_rect;

    return out;
}

// Signed distance function for a 2D rounded rectangle with per-corner radii
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    // Select radius depending on quadrant:
    // r.x = top_left, r.y = top_right, r.z = bottom_right, r.w = bottom_left
    var radius: f32;
    if (p.x < 0.0) {
        if (p.y < 0.0) {
            radius = r.x; // Top-Left
        } else {
            radius = r.w; // Bottom-Left
        }
    } else {
        if (p.y < 0.0) {
            radius = r.y; // Top-Right
        } else {
            radius = r.z; // Bottom-Right
        }
    }

    // Clamp radius to prevent corner overlap
    let max_r = min(b.x, b.y);
    radius = clamp(radius, 0.0, max_r);

    let q = abs(p) - b + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Scissor / Clipping test
    if (in.clip_bounds.z > in.clip_bounds.x && in.clip_bounds.w > in.clip_bounds.y) {
        if (in.screen_pos.x < in.clip_bounds.x || in.screen_pos.x > in.clip_bounds.z ||
            in.screen_pos.y < in.clip_bounds.y || in.screen_pos.y > in.clip_bounds.w) {
            discard;
        }
    }

    let half_size = in.rect_size * 0.5;
    let center = half_size;
    let p = in.local_pos - center;

    // Evaluate Box SDF
    let dist = sd_rounded_box(p, half_size, in.corner_radii);
    let aa = fwidth(dist) * 0.7071;
    let box_alpha = clamp(0.5 - dist / max(aa, 0.001), 0.0, 1.0);

    // Evaluate Drop Shadow
    var shadow_acc = vec4<f32>(0.0);
    let shadow_color = in.shadow_color;
    let shadow_blur = in.shadow_params.z;
    let shadow_spread = in.shadow_params.w;
    let shadow_offset = in.shadow_params.xy;

    if (shadow_color.a > 0.001 && (shadow_blur > 0.0 || shadow_spread > 0.0)) {
        let shadow_p = (in.local_pos - shadow_offset) - center;
        let shadow_dist = sd_rounded_box(shadow_p, half_size + vec2<f32>(shadow_spread), in.corner_radii + vec4<f32>(shadow_spread));
        
        let blur_factor = max(shadow_blur, 1.0);
        let s_alpha = clamp(0.5 - shadow_dist / blur_factor, 0.0, 1.0);
        let soft_alpha = s_alpha * s_alpha * (3.0 - 2.0 * s_alpha); // Smoothstep curve
        shadow_acc = vec4<f32>(shadow_color.rgb, shadow_color.a * soft_alpha);
    }

    // Evaluate Border
    let avg_border = (in.border_width.x + in.border_width.y + in.border_width.z + in.border_width.w) * 0.25;
    var final_color: vec4<f32>;

    if (avg_border > 0.001 && in.border_color.a > 0.001) {
        let inner_dist = dist + avg_border;
        let inner_alpha = clamp(0.5 - inner_dist / max(aa, 0.001), 0.0, 1.0);
        let border_factor = clamp(box_alpha - inner_alpha, 0.0, 1.0);

        let border_a = in.border_color.a * border_factor;
        let bg_a = in.color.a * inner_alpha;
        let composite_a = border_a + bg_a * (1.0 - border_a);

        let composite_rgb = (in.border_color.rgb * border_a + in.color.rgb * bg_a * (1.0 - border_a)) / max(composite_a, 0.0001);
        final_color = vec4<f32>(composite_rgb, composite_a);
    } else {
        final_color = vec4<f32>(in.color.rgb, in.color.a * box_alpha);
    }

    // Composite Box over Drop Shadow
    var out_color: vec4<f32>;
    if (shadow_acc.a > 0.001) {
        let out_a = final_color.a + shadow_acc.a * (1.0 - final_color.a);
        let out_rgb = (final_color.rgb * final_color.a + shadow_acc.rgb * shadow_acc.a * (1.0 - final_color.a)) / max(out_a, 0.0001);
        out_color = vec4<f32>(out_rgb, out_a);
    } else {
        out_color = final_color;
    }

    if (out_color.a < 0.001) {
        discard;
    }

    return out_color;
}
