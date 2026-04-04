mod climbing;
mod player;

use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, CollisionLayers, Friction, PhysicsLayer, RigidBody},
};
use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    asset::Assets,
    camera::ClearColor,
    color::Color,
    ecs::system::{Commands, ResMut, Single},
    light::PointLight,
    math::{EulerRot, Quat, primitives::Cuboid, vec3},
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    transform::components::Transform,
    utils::default,
    window::{CursorGrabMode, CursorOptions},
};

use crate::player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), PlayerPlugin))
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_systems(Startup, (core_setup, setup_map))
        .run();
}

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayers {
    #[default]
    Default,
    Ground,
}

fn core_setup(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}
fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let general_mat = materials.add(Color::srgb_u8(124, 144, 255));
    let ledge_mat = materials.add(Color::srgb_u8(200, 100, 100));

    commands.spawn((
        RigidBody::Static,
        Friction::new(0.0),
        Collider::cuboid(25.0, 0.5, 25.0),
        Mesh3d(meshes.add(Cuboid::from_size(vec3(25.0, 0.5, 25.0)))),
        MeshMaterial3d(general_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CollisionLayers::new(
            GameLayers::Ground,
            [GameLayers::Default, GameLayers::Ground],
        ),
    ));

    commands.spawn((
        RigidBody::Static,
        Friction::new(0.0),
        Collider::cuboid(5.0, 10.0, 1.0),
        Mesh3d(meshes.add(Cuboid::from_size(vec3(5.0, 10.0, 1.0)))),
        MeshMaterial3d(ledge_mat.clone()),
        Transform::from_xyz(0.0, 5.0, 5.0),
        CollisionLayers::new(
            GameLayers::Ground,
            [GameLayers::Default, GameLayers::Ground],
        ),
    ));

    commands.spawn((
        RigidBody::Static,
        Friction::new(0.0),
        Collider::cuboid(2.5, 4.0, 2.5),
        Mesh3d(meshes.add(Cuboid::from_size(vec3(2.5, 4.0, 2.5)))),
        MeshMaterial3d(ledge_mat.clone()),
        Transform::from_xyz(5.0, 2.0, 0.0),
        CollisionLayers::new(
            GameLayers::Ground,
            [GameLayers::Default, GameLayers::Ground],
        ),
    ));

    commands.spawn((
        RigidBody::Static,
        Friction::new(0.0),
        Collider::cuboid(2.5, 4.0, 2.5),
        Mesh3d(meshes.add(Cuboid::from_size(vec3(2.5, 4.0, 2.5)))),
        MeshMaterial3d(ledge_mat.clone()),
        Transform::from_xyz(7.5, 2.0, 2.5),
        CollisionLayers::new(
            GameLayers::Ground,
            [GameLayers::Default, GameLayers::Ground],
        ),
    ));

    for y in 0..12 {
        commands.spawn((
            RigidBody::Static,
            Friction::new(0.0),
            Collider::cuboid(2.0, 0.25, 0.5),
            Mesh3d(meshes.add(Cuboid::from_size(vec3(2.0, 0.25, 0.5)))),
            MeshMaterial3d(general_mat.clone()),
            Transform::from_xyz(0.0, 4.0 + y as f32 * 0.5, 4.7),
            CollisionLayers::new(
                GameLayers::Ground,
                [GameLayers::Default, GameLayers::Ground],
            ),
        ));
    }

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 0.0),
    ));
}
