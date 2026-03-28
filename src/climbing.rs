use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    ecs::{query::With, system::Single},
    math::{Dir3, Quat, USizeVec2, Vec2, Vec3, usizevec2, vec2, vec3},
    transform::components::Transform,
};

use crate::{
    GameLayers,
    player::{PlayerBody, lerp_f32},
};

pub const HAND_HOLD_DETECTION_CONFIG_FORWARD: HandHoldDetectionConfig =
    HandHoldDetectionConfig::new(
        2.0,
        0.01,
        0.5,
        usizevec2(9, 25),
        vec2(2.0, 1.75),
        vec3(0.0, 0.75, 0.5),
    );

fn get_hand_holds_forward(
    query: SpatialQuery,
    transform: &Single<&Transform, With<PlayerBody>>,
) -> Vec<HandHold> {
    let mut hand_holds = vec![];

    hand_holds
}
pub fn generate_hand_holds(
    config: HandHoldDetectionConfig,
    pos: Vec3,
    rot: Quat,
    query: SpatialQuery,
) -> Vec<HandHold> {
    let mut hand_holds = vec![];

    let origins = get_ray_origins(config, pos, rot);

    let down = rot * Dir3::NEG_Y;
    let forward = rot * Dir3::NEG_Z;

    let filter = SpatialQueryFilter::default().with_mask(GameLayers::Ground);

    for x in 0..config.ray_amount.x {
        for y in 0..config.ray_amount.y - 1 {
            let origin1 = origins[y][x];
            let dist1 = config.ray_length_forward;

            if let Some(hit1) = query.cast_ray(origin1, forward, dist1, true, &filter) {
                let origin2 = origins[y + 1][x];
                let dist2 = hit1.distance + config.min_ledge_depth;

                if query
                    .cast_ray(origin2, forward, dist2, true, &filter)
                    .is_none()
                {
                    let origin3 = origin2 + forward * dist2;
                    let dist3 = config.ray_length_down;

                    if let Some(hit2) = query.cast_ray(origin3, down, dist3, true, &filter) {
                        let hit1_pos = origin1 + forward * hit1.distance;
                        let hit2_pos = origin3 + down * hit2.distance;

                        let pos = vec3(hit1_pos.x, hit2_pos.y, hit1_pos.z);
                        let forward_normal = hit1.normal;
                        let up_normal = hit2.normal;

                        let hand_hold = HandHold::new(pos, forward_normal, up_normal);

                        hand_holds.push(hand_hold);
                    }
                }
            }
        }
    }

    hand_holds
}
fn get_ray_origins(config: HandHoldDetectionConfig, pos: Vec3, rot: Quat) -> Vec<Vec<Vec3>> {
    let mut origins =
        vec![vec![Vec3::ZERO; config.ray_amount.x as usize]; config.ray_amount.y as usize];

    let half_width = config.ray_bounds.x / 2.0;
    let half_height = config.ray_bounds.y / 2.0;

    let right = rot * Vec3::X;
    let up = rot * Vec3::Y;
    let forward = rot * Vec3::NEG_Z;

    for x in 0..config.ray_amount.x {
        for y in 0..config.ray_amount.y {
            let tx = x as f32 / config.ray_amount.x as f32;
            let ty = y as f32 / config.ray_amount.y as f32;

            let offset_x = lerp_f32(-half_width, half_width, tx);
            let offset_y = lerp_f32(-half_height, half_height, ty);

            let pos_x = right * (config.ray_offset.x + offset_x);
            let pos_y = up * (config.ray_offset.y + offset_y);
            let pos_z = forward * config.ray_offset.z;

            let origin = pos + pos_x + pos_y + pos_z;

            origins[y][x] = origin;
        }
    }

    origins
}
pub struct HandHold {
    pub pos: Vec3,
    pub forward_normal: Vec3,
    pub up_normal: Vec3,
}
impl HandHold {
    pub fn new(pos: Vec3, forward_normal: Vec3, up_normal: Vec3) -> Self {
        Self {
            pos,
            forward_normal,
            up_normal,
        }
    }
}
#[derive(Clone, Copy)]
pub struct HandHoldDetectionConfig {
    pub ray_length_forward: f32,
    pub min_ledge_depth: f32,
    pub ray_length_down: f32,

    pub ray_amount: USizeVec2,
    pub ray_bounds: Vec2,
    pub ray_offset: Vec3,
}
impl HandHoldDetectionConfig {
    pub const fn new(
        ray_length_forward: f32,
        min_ledge_depth: f32,
        ray_length_down: f32,

        ray_amount: USizeVec2,
        ray_bounds: Vec2,
        ray_offset: Vec3,
    ) -> Self {
        Self {
            ray_length_forward,
            min_ledge_depth,
            ray_length_down,
            ray_amount,
            ray_bounds,
            ray_offset,
        }
    }
}
