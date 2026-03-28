use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionLayers, Friction, LinearVelocity, RigidBody, SpatialQuery,
    SpatialQueryFilter,
};
use bevy::{
    app::{App, Plugin, Startup, Update},
    camera::Camera3d,
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        schedule::IntoScheduleConfigs,
        system::{Commands, Res, Single},
    },
    gizmos::gizmos::Gizmos,
    input::{ButtonInput, keyboard::KeyCode, mouse::AccumulatedMouseMotion},
    math::{Dir3, EulerRot, Isometry3d, Quat, Vec2, Vec3, vec2, vec3},
    time::Time,
    transform::components::Transform,
};

use crate::{
    GameLayers,
    climbing::{HAND_HOLD_DETECTION_CONFIG_FORWARD, generate_hand_holds},
};

pub const CAM_SENSITIVITY: f32 = 0.002;
pub const CAM_CLAMP_MIN: f32 = -PI / 2.0;
pub const CAM_CLAMP_MAX: f32 = PI / 2.0;

pub const JUMP_FORCE: f32 = 10.0;

pub const IDLE_HM_PARAMS: HorizontalMovementParams = HorizontalMovementParams::new(0.0, 10.0, 15.0);
pub const WALK_HM_PARAMS: HorizontalMovementParams = HorizontalMovementParams::new(5.0, 10.0, 15.0);
pub const FALL_HM_PARAMS: HorizontalMovementParams = HorizontalMovementParams::new(5.0, 10.0, 15.0);

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(
            Update,
            (
                update_input_dir,
                move_player_cam,
                rotate_player_cam,
                idle.run_if(is_player_state(PlayerMovementState::Idle)),
                walk.run_if(is_player_state(PlayerMovementState::Walking)),
                fall.run_if(is_player_state(PlayerMovementState::Falling)),
                draw_gizmos,
            ),
        );
    }
}

#[derive(Bundle)]
pub struct PlayerBodyBundle {
    tag: PlayerBody,
    rb: RigidBody,
    linear_vel: LinearVelocity,
    friction: Friction,
    collider: Collider,
    transform: Transform,
    layers: CollisionLayers,
    state: PlayerMovementStateComponent,
    input_dir: InputDir,
}

#[derive(Bundle)]
pub struct PlayerCamBundle {
    tag: PlayerCam,
    cam: Camera3d,
    transform: Transform,
    rot: PlayerCamRot,
}

#[derive(Component)]
struct PlayerCam;

#[derive(Component)]
pub struct PlayerBody;

#[derive(Component)]
struct PlayerCamRot(Vec2);

#[derive(Component)]
struct PlayerMovementStateComponent(PlayerMovementState);

#[derive(Component)]
struct InputDir(Vec2);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerMovementState {
    Idle,
    Walking,
    Falling,
}
fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBodyBundle {
        tag: PlayerBody,
        rb: RigidBody::Dynamic,
        linear_vel: LinearVelocity(vec3(0.0, 0.0, 0.0)),
        friction: Friction::new(0.0),
        collider: Collider::capsule(0.5, 2.0),
        transform: Transform::from_xyz(0.0, 15.0, 0.0),
        layers: CollisionLayers::new(GameLayers::Default, GameLayers::Ground),
        state: PlayerMovementStateComponent(PlayerMovementState::Idle),
        input_dir: InputDir(Vec2::ZERO),
    });

    commands.spawn(PlayerCamBundle {
        tag: PlayerCam,
        cam: Camera3d::default(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        rot: PlayerCamRot(Vec2::ZERO),
    });
}
fn move_player_cam(
    mut cam_transform: Single<&mut Transform, With<PlayerCam>>,
    body_transform: Single<&Transform, (With<PlayerBody>, Without<PlayerCam>)>,
) {
    cam_transform.translation = body_transform.translation + Vec3::Y * 1.5;
}
fn rotate_player_cam(
    mut cam_transform: Single<&mut Transform, With<PlayerCam>>,
    mut body_transform: Single<&mut Transform, (With<PlayerBody>, Without<PlayerCam>)>,
    mut cam_rot: Single<&mut PlayerCamRot, With<PlayerCam>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let look = vec2(
        -mouse_motion.delta.y * CAM_SENSITIVITY,
        -mouse_motion.delta.x * CAM_SENSITIVITY,
    );

    cam_rot.0 += look;
    cam_rot.0.x = cam_rot.0.x.clamp(CAM_CLAMP_MIN, CAM_CLAMP_MAX);

    cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, cam_rot.0.y, cam_rot.0.x, 0.0);
    body_transform.rotation = Quat::from_euler(EulerRot::YXZ, cam_rot.0.y, 0.0, 0.0);
}
fn is_grounded(query: &SpatialQuery, transform: &Single<&Transform, With<PlayerBody>>) -> bool {
    let origin = transform.translation - Vec3::Y;
    let dir = Dir3::NEG_Y;
    let length = 0.7;

    query
        .cast_ray(
            origin,
            dir,
            length,
            true,
            &SpatialQueryFilter::default().with_mask(GameLayers::Ground),
        )
        .is_some()
}
fn is_player_state(
    check: PlayerMovementState,
) -> impl Fn(Single<&PlayerMovementStateComponent, With<PlayerBody>>) -> bool {
    move |single: Single<&PlayerMovementStateComponent, With<PlayerBody>>| single.0 == check
}
fn idle(
    query: SpatialQuery,
    transform: Single<&Transform, With<PlayerBody>>,
    input_dir: Single<&InputDir, With<PlayerBody>>,
    mut state: Single<&mut PlayerMovementStateComponent, With<PlayerBody>>,
    input: Res<ButtonInput<KeyCode>>,
    mut linear_vel: Single<&mut LinearVelocity, With<PlayerBody>>,
    time: Res<Time>,
) {
    move_horizontal(
        IDLE_HM_PARAMS,
        &transform,
        &input_dir,
        &mut linear_vel,
        time,
    );

    if !is_grounded(&query, &transform) {
        state.0 = PlayerMovementState::Falling;
        return;
    }

    if is_grounded(&query, &transform) && input.just_pressed(KeyCode::Space) {
        linear_vel.0.y = JUMP_FORCE;
        state.0 = PlayerMovementState::Falling;
        return;
    }

    if input_dir.0.length_squared() > 0.1 {
        state.0 = PlayerMovementState::Walking;
        return;
    }
}
fn walk(
    query: SpatialQuery,
    transform: Single<&Transform, With<PlayerBody>>,
    input_dir: Single<&InputDir, With<PlayerBody>>,
    mut state: Single<&mut PlayerMovementStateComponent, With<PlayerBody>>,
    input: Res<ButtonInput<KeyCode>>,
    mut linear_vel: Single<&mut LinearVelocity, With<PlayerBody>>,
    time: Res<Time>,
) {
    move_horizontal(
        WALK_HM_PARAMS,
        &transform,
        &input_dir,
        &mut linear_vel,
        time,
    );

    if !is_grounded(&query, &transform) {
        state.0 = PlayerMovementState::Falling;
        return;
    }

    if is_grounded(&query, &transform) && input.just_pressed(KeyCode::Space) {
        linear_vel.0.y = JUMP_FORCE;
        state.0 = PlayerMovementState::Falling;
        return;
    }

    if input_dir.0.length_squared() < 0.1 {
        state.0 = PlayerMovementState::Idle;
        return;
    }
}
fn fall(
    query: SpatialQuery,
    transform: Single<&Transform, With<PlayerBody>>,
    input_dir: Single<&InputDir, With<PlayerBody>>,
    mut state: Single<&mut PlayerMovementStateComponent, With<PlayerBody>>,
    mut linear_vel: Single<&mut LinearVelocity, With<PlayerBody>>,
    time: Res<Time>,
) {
    move_horizontal(
        FALL_HM_PARAMS,
        &transform,
        &input_dir,
        &mut linear_vel,
        time,
    );

    if is_grounded(&query, &transform) {
        state.0 = PlayerMovementState::Idle;
        return;
    }
}
fn update_input_dir(
    input: Res<ButtonInput<KeyCode>>,
    mut input_dir: Single<&mut InputDir, With<PlayerBody>>,
) {
    let mut x = 0.0;

    if input.pressed(KeyCode::KeyD) {
        x += 1.0;
    }

    if input.pressed(KeyCode::KeyA) {
        x -= 1.0;
    }

    let mut y = 0.0;

    if input.pressed(KeyCode::KeyW) {
        y += 1.0;
    }

    if input.pressed(KeyCode::KeyS) {
        y -= 1.0;
    }

    let mut final_dir = vec2(x, y);

    if final_dir.length_squared() != 0.0 {
        final_dir = final_dir.normalize();
    }

    input_dir.0 = final_dir;
}
fn move_horizontal(
    params: HorizontalMovementParams,
    transform: &Single<&Transform, With<PlayerBody>>,
    input_dir: &Single<&InputDir, With<PlayerBody>>,
    linear_vel: &mut Single<&mut LinearVelocity, With<PlayerBody>>,
    time: Res<Time>,
) {
    let move_dir = transform.right() * input_dir.0.x + transform.forward() * input_dir.0.y;

    let current_hv = vec3(linear_vel.x, 0.0, linear_vel.z);

    let target_hv = move_dir * params.speed;

    let lerp_rate = if current_hv.length_squared() > target_hv.length_squared() {
        params.brake_rate
    } else {
        params.accel_rate
    };

    let lerped_vel = lerp_vec3(current_hv, target_hv, lerp_rate * time.delta_secs());

    linear_vel.0.x = lerped_vel.x;
    linear_vel.0.z = lerped_vel.z;
}
#[derive(Clone, Copy)]
pub struct HorizontalMovementParams {
    speed: f32,
    accel_rate: f32,
    brake_rate: f32,
}
impl HorizontalMovementParams {
    pub const fn new(speed: f32, accel_rate: f32, brake_rate: f32) -> Self {
        Self {
            speed,
            accel_rate,
            brake_rate,
        }
    }
}
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let lerped_x = lerp_f32(a.x, b.x, t);
    let lerped_y = lerp_f32(a.y, b.y, t);
    let lerped_z = lerp_f32(a.z, b.z, t);

    vec3(lerped_x, lerped_y, lerped_z)
}

fn draw_gizmos(
    transform: Single<&Transform, With<PlayerBody>>,
    mut gizmos: Gizmos,
    query: SpatialQuery,
) {
    let points = generate_hand_holds(
        HAND_HOLD_DETECTION_CONFIG_FORWARD,
        transform.translation,
        transform.rotation,
        query,
    );

    for point in points {
        gizmos.sphere(
            Isometry3d::from_translation(point.pos),
            0.1,
            Color::srgb_u8(255, 0, 0),
        );
    }
}
