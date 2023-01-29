#![warn(
missing_debug_implementations,
// rust_2018_idioms,
// missing_docs,
)]

// use bevy::log::LogPlugin;
use bevy::{
    prelude::*,
};

use crate::camera::*;
use crate::lines::{LineMaterial, setup_cylinders, setup_lines};
use crate::scene_setup::*;

mod scene_setup;
mod camera;
mod lines;
mod cylinder;

fn main() {

    print_help();

    let ambient_light = AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        };

    let mut app = App::new();

    app.insert_resource(ambient_light)
        .init_resource::<CameraTracker>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "bevy scene viewer".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .set(AssetPlugin {
                    asset_folder: std::env::var("CARGO_MANIFEST_DIR")
                        .unwrap_or_else(|_| ".".to_string()),
                    watch_for_changes: true,
                })
               // .build().disable::<LogPlugin>()
        )
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_startup_system(setup_scene)
        .add_startup_system(setup_lines)
        .add_startup_system(setup_cylinders)
        .add_system_to_stage(CoreStage::PreUpdate, scene_load_check)
        .add_system_to_stage(CoreStage::PreUpdate, setup_scene_after_load)
        .add_system(update_lights)
        .add_system(camera_controller)
        // .add_system(camera_tracker)
    ;

    #[cfg(feature = "animation")]
    app.add_system(start_animation)
        .add_system(keyboard_animation_control);

    // bevy_mod_debugdump::print_render_graph(&mut app);

    app.run();
}

fn print_help() {
    println!(
        "
Controls:
    MOUSE       - Move camera orientation
    LClick/M    - Enable mouse movement
    WSAD        - forward/back/strafe left/right
    LShift      - 'run'
    E           - up
    Q           - down
    L           - animate light direction
    U           - toggle shadows
    C           - cycle through the camera controller and any cameras loaded from the scene
    5/6         - decrease/increase shadow projection width
    7/8         - decrease/increase shadow projection height
    9/0         - decrease/increase shadow projection near/far

    Space       - Play/Pause animation
    Enter       - Cycle through animations
"
    );
}