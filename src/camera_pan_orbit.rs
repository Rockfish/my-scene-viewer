// use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel,MouseMotion};
use bevy::render::camera::Projection;
use bevy::window::*;

// ANCHOR: example
/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn pan_orbit_camera(
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    input_keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // if let Ok(primary) = primary_query.get_single();
    let primary = primary_query.get_single().unwrap();

    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let _pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    // if input_mouse.pressed(orbit_button) {
    if input_mouse.pressed(orbit_button) && !input_keyboard.pressed(KeyCode::LShift) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
        // } else if input_mouse.pressed(pan_button) {
    } else if input_mouse.pressed(orbit_button) && input_keyboard.pressed(KeyCode::LShift) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y * 0.05;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {

            any = true;

            let delta_x = {
                let delta = rotation_move.x / primary.width() * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down { -delta } else { delta }
            };

            let delta_y = rotation_move.y / primary.height() * std::f32::consts::PI;

            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);

            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis

        } else if pan.length_squared() > 0.0 {
            any = true;

            // make panning distance independent of resolution and FOV,
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(
                    (projection.fov * projection.aspect_ratio)/primary.width(),
                    projection.fov/primary.height()
                );
            }

            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;

            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;

        } else if scroll.abs() > 0.0 {
            any = true;

            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

pub fn spawn_camera(commands: &mut Commands) {
    info!("Spawning a controllable 3D perspective camera");

    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
}