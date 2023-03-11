//! A simple glTF scene viewer made with Bevy.
//!
//! Just run `cargo run --release --example scene_viewer /path/to/model.gltf#Scene0`,
//! replacing the path as appropriate.
//! With no arguments it will load the `FieldHelmet` glTF model from the repository assets subdirectory.

use std::f32::consts::PI;
use bevy::{
    asset::LoadState,
    gltf::Gltf,
    math::Vec3A,
    prelude::*,
    render::primitives::{Aabb, Sphere},
    scene::InstanceId,
};
use bevy::pbr::CascadeShadowConfigBuilder;

use crate::CameraController;

// #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
// struct CameraControllerCheckSystem;

#[derive(Resource)]
pub struct SceneHandle {
    handle: Handle<Gltf>,
    #[cfg(feature = "animation")]
    animations: Vec<Handle<AnimationClip>>,
    instance_id: Option<InstanceId>,
    is_loaded: bool,
    has_light: bool,
}

pub fn setup_scene(mut commands: Commands<'_, '_>, asset_server: Res<AssetServer>) {

    let scene_path = std::env::args()
        .nth(1)
        // .unwrap_or_else(|| "assets/models/FlightHelmet/FlightHelmet.gltf".to_string());
        // .unwrap_or_else(|| "assets/models/monkey/Monkey.gltf".to_string());
        // .unwrap_or_else(|| "assets/craft_speederA.gltf".to_string());
        .unwrap_or_else(|| "assets/models/alien.glb".to_string());

    info!("Loading {}", scene_path);

    commands.insert_resource(SceneHandle {
        handle: asset_server.load(&scene_path),
        #[cfg(feature = "animation")]
        animations: Vec::new(),
        instance_id: None,
        is_loaded: false,
        has_light: false,
    });
}

pub fn scene_load_check(
    asset_server: Res<AssetServer>,
    mut scenes: ResMut<Assets<Scene>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    mut scene_handle: ResMut<SceneHandle>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    match scene_handle.instance_id {
        None => {
            if asset_server.get_load_state(&scene_handle.handle) == LoadState::Loaded {
                let gltf = gltf_assets.get(&scene_handle.handle).unwrap();
                let gltf_scene_handle = gltf.scenes.first().expect("glTF file contains no scenes!");
                let scene = scenes.get_mut(gltf_scene_handle).unwrap();

                let mut query = scene
                    .world
                    .query::<(Option<&DirectionalLight>, Option<&PointLight>)>();

                scene_handle.has_light =
                    query
                        .iter(&scene.world)
                        .any(|(maybe_directional_light, maybe_point_light)| {
                            maybe_directional_light.is_some() || maybe_point_light.is_some()
                        });

                scene_handle.instance_id =
                    Some(scene_spawner.spawn(gltf_scene_handle.clone_weak()));

                #[cfg(feature = "animation")]
                {
                    scene_handle.animations = gltf.animations.clone();
                    if !scene_handle.animations.is_empty() {
                        info!(
                            "Found {} animation{}",
                            scene_handle.animations.len(),
                            if scene_handle.animations.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        );
                    }
                }

                info!("Spawning scene...");
            }
        }
        Some(instance_id) if !scene_handle.is_loaded => {
            if scene_spawner.instance_is_ready(instance_id) {
                info!("...done!");
                scene_handle.is_loaded = true;
            }
        }
        Some(_) => {}
    }
}

#[cfg(feature = "animation")]
fn start_animation(
    mut player: Query<&mut AnimationPlayer>,
    mut done: Local<bool>,
    scene_handle: Res<SceneHandle>,
) {
    if !*done {
        if let Ok(mut player) = player.get_single_mut() {
            if let Some(animation) = scene_handle.animations.first() {
                player.play(animation.clone_weak()).repeat();
                *done = true;
            }
        }
    }
}

#[cfg(feature = "animation")]
fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer>,
    scene_handle: Res<SceneHandle>,
    mut current_animation: Local<usize>,
    mut changing: Local<bool>,
) {
    if scene_handle.animations.is_empty() {
        return;
    }

    if let Ok(mut player) = animation_player.get_single_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }

        if *changing {
            // change the animation the frame after return was pressed
            *current_animation = (*current_animation + 1) % scene_handle.animations.len();
            player
                .play(scene_handle.animations[*current_animation].clone_weak())
                .repeat();
            *changing = false;
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            // delay the animation change for one frame
            *changing = true;
            // set the current animation to its start and pause it to reset to its starting state
            player.set_elapsed(0.0).pause();
        }
    }
}

pub fn setup_scene_after_load(
    mut commands: Commands,
    mut setup: Local<bool>,
    mut scene_handle: ResMut<SceneHandle>,
    meshes: Query<(&GlobalTransform, Option<&Aabb>), With<Handle<Mesh>>>,
) {
    if scene_handle.is_loaded && !*setup {
        *setup = true;
        // Find an approximate bounding box of the scene from its meshes
        if meshes.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
            return;
        }

        let mut min = Vec3A::splat(f32::MAX);
        let mut max = Vec3A::splat(f32::MIN);

        for (transform, maybe_aabb) in &meshes {
            let aabb = maybe_aabb.unwrap();
            // If the Aabb had not been rotated, applying the non-uniform scale would produce the
            // correct bounds. However, it could very well be rotated and so we first convert to
            // a Sphere, and then back to an Aabb to find the conservative min and max points.
            let sphere = Sphere {
                center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
                radius: transform.radius_vec3a(aabb.half_extents),
            };
            let aabb = Aabb::from(sphere);
            min = min.min(aabb.min());
            max = max.max(aabb.max());
        }

        let size = (max - min).length();
        let aabb = Aabb::from_min_max(Vec3::from(min), Vec3::from(max));

        info!("Spawning a controllable 3D perspective camera");

        let mut projection = PerspectiveProjection::default();

        projection.far = projection.far.max(size * 10.0);

        let mut camera_controller = CameraController::default();
        camera_controller.target = Vec3::from(aabb.center);

        commands.spawn((
            Camera3dBundle {
                projection: projection.into(),
                transform: Transform::from_translation(
                    Vec3::from(aabb.center) + size * Vec3::new(0.5, 0.25, 0.5),
                ).looking_at(Vec3::from(aabb.center), Vec3::Y),
                camera: Camera {
                    is_active: true,
                    ..default()
                },
                ..default()
            },
            camera_controller,
        ));

        // Spawn a default light if the scene does not have one
        if !scene_handle.has_light {
            let sphere = Sphere {
                center: aabb.center,
                radius: aabb.half_extents.length(),
            };
            let aabb = Aabb::from(sphere);
            let min = aabb.min();
            let max = aabb.max();

            info!("Spawning a directional light");
            // commands.spawn(DirectionalLightBundle {
            //     directional_light: DirectionalLight {
            //         shadow_projection: OrthographicProjection {
            //             left: min.x,
            //             right: max.x,
            //             bottom: min.y,
            //             top: max.y,
            //             near: min.z,
            //             far: max.z,
            //             ..default()
            //         },
            //         shadows_enabled: false,
            //         ..default()
            //     },
            //     ..default()
            // });

            // directional 'sun' light
            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 2.0, 0.0),
                    rotation: Quat::from_rotation_x(-PI / 4.),
                    ..default()
                },
                // The default cascade config is designed to handle large scenes.
                // As this example has a much smaller world, we can tighten the shadow
                // bounds for better visual quality.
                cascade_shadow_config: CascadeShadowConfigBuilder {
                    first_cascade_far_bound: 4.0,
                    maximum_distance: 10.0,
                    ..default()
                }
                    .into(),
                ..default()
            });

            scene_handle.has_light = true;
        }
    }
}

