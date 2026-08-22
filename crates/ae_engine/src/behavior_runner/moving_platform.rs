// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Moving platform and elevator subsystem with waypoint ping-pong interpolation.
//!

use ae_core::ecs::{MovingPlatform, Position};
use hecs::{Entity, World};

/// Updates waypoint interpolation for moving platforms and translates any standing passengers.
pub fn update_moving_platforms(
    world: &mut World,
    player_entity_and_pos: Option<(Entity, [f32; 3])>,
    dt: f32,
    dirty_entities: &mut Vec<Entity>,
) {
    let mut player_delta: Option<(Entity, [f32; 3])> = None;

    for (ent, pos, platform) in world.query_mut::<(Entity, &mut Position, &mut MovingPlatform)>() {
        let target = if platform.ping_pong_forward {
            platform.target_position
        } else {
            platform.original_position
        };
        let speed = platform.speed;

        let dx = target[0] - pos.x;
        let dy = target[1] - pos.y;
        let dz = target[2] - pos.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 0.05 {
            // Reached waypoint: flip direction
            platform.ping_pong_forward = !platform.ping_pong_forward;
        } else {
            let step = (speed * dt).min(dist);
            let inv_dist = 1.0 / dist;
            let step_x = dx * inv_dist * step;
            let step_y = dy * inv_dist * step;
            let step_z = dz * inv_dist * step;

            // If player is standing on the elevator/platform, carry player
            if let Some((p_ent, p_pos)) = player_entity_and_pos {
                let on_platform_xz =
                    (p_pos[0] - pos.x).abs() <= 2.3 && (p_pos[2] - pos.z).abs() <= 2.3;
                let on_platform_y = p_pos[1] >= pos.y - 0.2 && p_pos[1] <= pos.y + 1.8;
                if on_platform_xz && on_platform_y {
                    player_delta = Some((p_ent, [step_x, step_y, step_z]));
                }
            }

            pos.x += step_x;
            pos.y += step_y;
            pos.z += step_z;
            dirty_entities.push(ent);
        }
    }

    if let Some((p_ent, [step_x, step_y, step_z])) = player_delta
        && let Ok(mut player_pos_comp) = world.get::<&mut Position>(p_ent)
    {
        player_pos_comp.x += step_x;
        player_pos_comp.y += step_y;
        player_pos_comp.z += step_z;
        dirty_entities.push(p_ent);
    }
}