use avian3d::prelude::{Collider, SpatialQuery, SpatialQueryFilter};
use bevy::{
    ecs::{query::With, system::Single},
    math::{Dir3, Quat, USizeVec2, Vec2, Vec3, usizevec2, vec2, vec3},
    transform::components::Transform,
};

use crate::{
    GameLayers,
    player::{PlayerBody, get_player_pos_relative_to_hand_hold, lerp_f32},
};

pub const PLAYER_CLIMB_POS_OFFSET: Vec3 = vec3(0.0, 1.75, 1.0);
pub const PLAYER_CLIMB_POS_LERP_RATE: f32 = 5.0;
pub const PLAYER_CLIMB_ROT_LERP_RATE: f32 = 5.0;

pub const MAX_CLIMB_START_DIST: f32 = 0.5;
pub const MAX_CLIMB_VERTICAL_XZ_DIST: f32 = 0.5;
pub const MAX_CLIMB_SIDE_Y_DIST: f32 = 0.1;

#[derive(Clone, Copy)]
pub enum ClimbType {
    Start,
    Up,
    Down,
    Right,
    Left,
}

pub const HAND_HOLD_DETECTION_CONFIG_FORWARD: HandHoldDetectionConfig =
    HandHoldDetectionConfig::new(
        3.0,
        0.01,
        0.5,
        usizevec2(9, 25),
        vec2(1.0, 1.75),
        vec3(0.0, 1.75, 0.0),
    );

pub fn get_hand_hold<'a>(
    hand_holds: &'a Vec<HandHold>,
    transform: &Single<&Transform, With<PlayerBody>>,
    climb_type: ClimbType,
) -> Option<&'a HandHold> {
    match climb_type {
        ClimbType::Start => get_hand_hold_start(hand_holds, transform),
        ClimbType::Up => get_hand_hold_vertical(hand_holds, transform, true),
        ClimbType::Down => get_hand_hold_vertical(hand_holds, transform, false),
        ClimbType::Right => get_hand_hold_side(hand_holds, transform, true),
        ClimbType::Left => get_hand_hold_side(hand_holds, transform, false),
    }
}
fn get_hand_hold_start<'a>(
    hand_holds: &'a Vec<HandHold>,
    transform: &Single<&Transform, With<PlayerBody>>,
) -> Option<&'a HandHold> {
    let mut best_hand_hold = None;
    let mut best_score = f32::MIN;

    let player_hold_pos = transform.translation
        + transform.right() * PLAYER_CLIMB_POS_OFFSET.x
        + transform.up() * PLAYER_CLIMB_POS_OFFSET.y
        + transform.forward() * PLAYER_CLIMB_POS_OFFSET.z;

    for hand_hold in hand_holds {
        let dist = (hand_hold.pos - player_hold_pos).length();

        if dist > MAX_CLIMB_START_DIST {
            continue;
        }

        let score = -dist;

        if score > best_score {
            best_score = score;
            best_hand_hold = Some(hand_hold);
        }
    }

    best_hand_hold
}
fn get_hand_hold_vertical<'a>(
    hand_holds: &'a Vec<HandHold>,
    transform: &Single<&Transform, With<PlayerBody>>,
    up: bool,
) -> Option<&'a HandHold> {
    let mut best_hand_hold = None;
    let mut best_score = f32::MIN;

    let player_hold_pos = transform.translation
        + transform.right() * PLAYER_CLIMB_POS_OFFSET.x
        + transform.up() * PLAYER_CLIMB_POS_OFFSET.y
        + transform.forward() * PLAYER_CLIMB_POS_OFFSET.z;

    for hand_hold in hand_holds {
        let dist_xz = -(vec3(hand_hold.pos.x, 0.0, hand_hold.pos.z)
            - vec3(player_hold_pos.x, 0.0, player_hold_pos.z))
        .length();

        if dist_xz > MAX_CLIMB_VERTICAL_XZ_DIST {
            continue;
        }

        let dist_y = if up {
            hand_hold.pos.y - player_hold_pos.y
        } else {
            player_hold_pos.y - hand_hold.pos.y
        };

        let score = dist_xz + dist_y;

        if score > best_score {
            best_score = score;
            best_hand_hold = Some(hand_hold);
        }
    }

    best_hand_hold
}
fn get_hand_hold_side<'a>(
    hand_holds: &'a Vec<HandHold>,
    transform: &Single<&Transform, With<PlayerBody>>,
    right: bool,
) -> Option<&'a HandHold> {
    let mut best_hand_hold = None;
    let mut best_score = f32::MIN;

    let player_hold_pos = transform.translation
        + transform.right() * PLAYER_CLIMB_POS_OFFSET.x
        + transform.up() * PLAYER_CLIMB_POS_OFFSET.y
        + transform.forward() * PLAYER_CLIMB_POS_OFFSET.z;

    for hand_hold in hand_holds {
        let dist_y = (hand_hold.pos.y - player_hold_pos.y).abs();

        if dist_y > MAX_CLIMB_SIDE_Y_DIST {
            continue;
        }

        let dir = (hand_hold.pos - player_hold_pos).normalize();

        let dot = transform.right().dot(dir);

        let valid = if right { dot > 0.1 } else { dot < 0.1 };

        if !valid {
            continue;
        }

        let dist_xz = (vec3(hand_hold.pos.x, 0.0, hand_hold.pos.z)
            - vec3(player_hold_pos.x, 0.0, player_hold_pos.z))
        .length();

        let score = dist_xz - dist_y;

        if score > best_score {
            best_score = score;
            best_hand_hold = Some(hand_hold);
        }
    }

    best_hand_hold
}
pub fn generate_hand_holds(
    config: HandHoldDetectionConfig,
    pos: Vec3,
    rot: Quat,
    query: &SpatialQuery,
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

                        if is_hand_hold_valid(&hand_hold, query) {
                            hand_holds.push(hand_hold);
                        }
                    }
                }
            }
        }
    }

    hand_holds
}
fn is_hand_hold_valid(hand_hold: &HandHold, query: &SpatialQuery) -> bool {
    let filter = SpatialQueryFilter::default().with_mask(GameLayers::Ground);

    let check_1 = query
        .shape_intersections(
            &Collider::capsule(0.5, 2.0),
            get_player_pos_relative_to_hand_hold(hand_hold),
            Quat::IDENTITY,
            &filter,
        )
        .len()
        == 0;

    check_1
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
            let tx = x as f32 / (config.ray_amount.x - 1) as f32;
            let ty = y as f32 / (config.ray_amount.y - 1) as f32;

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
#[derive(Clone, Copy)]
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
pub fn hand_holds_similar(a: &HandHold, b: &HandHold) -> bool {
    (a.pos - b.pos).length_squared() < 0.1
}
