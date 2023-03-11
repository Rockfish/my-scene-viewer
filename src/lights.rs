use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{
    prelude::*,
};

const SCALE_STEP: f32 = 0.1;

pub fn update_lights(
    key_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut DirectionalLight)>,
    mut animate_directional_light: Local<bool>,
) {
    let mut projection_adjustment = Vec3::ONE;
    if key_input.just_pressed(KeyCode::Key5) {
        projection_adjustment.x -= SCALE_STEP;
    } else if key_input.just_pressed(KeyCode::Key6) {
        projection_adjustment.x += SCALE_STEP;
    } else if key_input.just_pressed(KeyCode::Key7) {
        projection_adjustment.y -= SCALE_STEP;
    } else if key_input.just_pressed(KeyCode::Key8) {
        projection_adjustment.y += SCALE_STEP;
    } else if key_input.just_pressed(KeyCode::Key9) {
        projection_adjustment.z -= SCALE_STEP;
    } else if key_input.just_pressed(KeyCode::Key0) {
        projection_adjustment.z += SCALE_STEP;
    }
    // for (_, mut light) in &mut query {
    //     light.shadow_projection.left *= projection_adjustment.x;
    //     light.shadow_projection.right *= projection_adjustment.x;
    //     light.shadow_projection.bottom *= projection_adjustment.y;
    //     light.shadow_projection.top *= projection_adjustment.y;
    //     light.shadow_projection.near *= projection_adjustment.z;
    //     light.shadow_projection.far *= projection_adjustment.z;
    //     if key_input.just_pressed(KeyCode::U) {
    //         light.shadows_enabled = !light.shadows_enabled;
    //     }
    // }

    if key_input.just_pressed(KeyCode::L) {
        *animate_directional_light = !*animate_directional_light;
    }
    if *animate_directional_light {
        for (mut transform, _) in &mut query {
            transform.rotation = Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                time.elapsed_seconds() * PI / 15.0,
                -FRAC_PI_4,
            );
        }
    }
}

