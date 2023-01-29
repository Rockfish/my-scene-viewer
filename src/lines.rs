use std::f32::consts::PI;
use bevy::{
    prelude::*,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use crate::cylinder::Cylinder;

pub fn setup_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    // Spawn a list of lines with start and end points for each lines
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)),
                (Vec3::new(1.0, 0.02, 0.0), Vec3::new(1.0, -0.02, 0.0)),
            ],
        })),
        material: materials.add(LineMaterial { color: Color::RED, }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0)),
            ],
        })),
        material: materials.add(LineMaterial { color: Color::GREEN, }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0)),
            ],
        })),
        material: materials.add(LineMaterial { color: Color::BLUE, }),
        ..default()
    });



    // // Spawn a line strip that goes from point to point
    // commands.spawn(MaterialMeshBundle {
    //     mesh: meshes.add(Mesh::from(LineStrip {
    //         points: vec![
    //             Vec3::ZERO,
    //             Vec3::new(1.0, 1.0, 0.0),
    //             Vec3::new(1.0, 0.0, 0.0),
    //         ],
    //     })),
    //     transform: Transform::from_xyz(0.5, 0.0, 0.0),
    //     material: materials.add(LineMaterial { color: Color::BLUE }),
    //     ..default()
    // });

    // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}

const X_EXTENT: f32 = 14.;

pub fn setup_cylinders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(
            Cylinder {
                radius: 0.04,
                height: 2.0,
                resolution: 20,
                segments: 10,
            }
        )),
        material: materials.add(Color::rgb(0.63, 0.96, 0.26).into()), // greenish - y up
        transform: Transform::from_xyz( 0.0, 1.0, 0.0, ),
            // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(
            Cylinder {
                radius: 0.04,
                height: 2.0,
                resolution: 20,
                segments: 10,
            }
        )),
        // material: materials.add(LineMaterial { color: Color::YELLOW, }),
        material: materials.add(Color::rgb(0.96, 0.20, 0.20).into()), // redish - x right
        transform: Transform::from_xyz( 1.0, 0.0, 0.0, )
            .with_rotation(Quat::from_rotation_z(PI / 2.)),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(
            Cylinder {
                radius: 0.04,
                height: 2.0,
                resolution: 20,
                segments: 10,
            }
        )),
        // material: materials.add(LineMaterial { color: Color::YELLOW, }),
        material: materials.add(Color::rgb(0.20, 0.20, 0.96).into()), // bluish z - out
        transform: Transform::from_xyz( 0.0, 0.0, 1.0, )
            .with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    });
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
pub struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "assets/shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}

// /// A list of points that will have a line drawn between each consecutive points
// #[derive(Debug, Clone)]
// pub struct LineStrip {
//     pub points: Vec<Vec3>,
// }
//
// impl From<LineStrip> for Mesh {
//     fn from(line: LineStrip) -> Self {
//         // This tells wgpu that the positions are a list of points
//         // where a line will be drawn between each consecutive point
//         let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
//
//         mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line.points);
//         mesh
//     }
// }
