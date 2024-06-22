use super::{
    print_analyzer::Label, ForceRefresh, GCode, IdMap, PickableBundle, Pos, Tag, UiResource,
};
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_asset::RenderAssetUsages;

pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<IdMap>,
    gcode: Res<GCode>,
    cylinders: Query<Entity, With<Tag>>,
) {
    for cylinder in cylinders.iter() {
        commands.entity(cylinder).despawn();
    }
    let gcode = &gcode.0;

    for (id, vertex) in gcode.vertices.iter() {
        let Pos {
            x: xf,
            y: yf,
            z: zf,
            ..
        } = vertex.to;
        let (xi, yi, zi) = {
            if let Some(prev) = vertex.prev {
                let p = gcode.vertices.get(&prev).unwrap();
                (p.to.x, p.to.y, p.to.z)
            } else {
                (0.0, 0.0, 0.0)
            }
        };

        let start = Vec3::new(xi, yi, zi);
        let end = Vec3::new(xf, yf, zf);

        // Create a cylinder mesh
        let radius = 0.05;
        let length = start.distance(end);
        let cylinder = Cylinder {
            radius,
            half_height: length / 2.0,
        };
        let sphere = Sphere {
            radius: radius * 1.618,
        };

        // Create the mesh and material
        //let mesh_handle = meshes.add(cylinder);
        let sphere = meshes.add(sphere);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 1.0, 0.0),
            emissive: Color::rgb(0.0, 1.0, 0.0),
            ..Default::default()
        });
        let material_handle2 = materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..Default::default()
        });

        // Add the move line to the scene
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    Mesh::new(
                        bevy::render::mesh::PrimitiveTopology::LineList,
                        RenderAssetUsages::RENDER_WORLD,
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::from([end, start])),
                ),
                //mesh: mesh_handle,
                material: material_handle,
                //transform: Transform {
                //    translation: middle,
                //    rotation,
                //    ..Default::default()
                //},
                ..Default::default()
            },
            Tag { id: *id },
        ));
        // add the
        let e_id = commands
            .spawn((
                PbrBundle {
                    mesh: sphere,
                    material: material_handle2,
                    transform: Transform {
                        translation: end,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PickableBundle::default(),
                Tag { id: *id },
            ))
            .id();
        map.0.insert(*id, e_id);
    }
    commands.remove_resource::<ForceRefresh>();
}

pub fn update_visibilities(
    mut entity_query: Query<(&Tag, &mut Visibility)>,
    ui_res: Res<UiResource>,
    gcode: Res<GCode>,
) {
    let count = ui_res.vertex_counter;
    for (tag, mut vis) in entity_query.iter_mut() {
        if let Some(v) = gcode.0.vertices.get(&tag.id) {
            let selected = match v.label {
                Label::PrePrintMove => ui_res.vis_select.preprint,
                Label::PlanarExtrustion | Label::NonPlanarExtrusion => ui_res.vis_select.extrusion,
                Label::Retraction => ui_res.vis_select.retraction,
                Label::DeRetraction => ui_res.vis_select.deretraction,
                Label::Wipe => ui_res.vis_select.wipe,
                Label::LiftZ | Label::TravelMove => ui_res.vis_select.travel,
                _ => false,
            };
            if count > v.count
                && selected
                && v.to.z < ui_res.display_z_max.0
                && v.to.z > ui_res.display_z_min
            {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}
