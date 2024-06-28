use super::{
    print_analyzer::Label, settings::*, ForceRefresh, GCode, IdMap, PickableBundle, Tag, UiResource,
};
use bevy::prelude::*;
use bevy_mod_picking::{
    focus::PickingInteraction, highlight::PickHighlight, selection::PickSelection,
};
use std::collections::HashSet;

pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<IdMap>,
    gcode: Res<GCode>,
    shapes: Query<Entity, With<Tag>>,
    settings: Res<Settings>,
) {
    for shape in shapes.iter() {
        commands.entity(shape).despawn();
    }
    let gcode = &gcode.0;
    let mut pos_list = Vec::new();
    for v in gcode.vertices.values() {
        let (xf, yf, zf) = (v.to.x, v.to.y, v.to.z);
        let (xi, yi, zi) = {
            if let Some(prev) = v.prev {
                let p = gcode.vertices.get(&prev).unwrap();
                (p.to.x, p.to.y, p.to.z)
            } else {
                (0.0, 0.0, 0.0)
            }
        };
        let (start, end) = (Vec3::new(xi, yi, zi), Vec3::new(xf, yf, zf));
        let dist = start.distance(end);
        let flow = v.to.e / dist;
        pos_list.push((v.id, start, end, flow, v.label));
    }
    for (id, start, end, flow, label) in pos_list {
        if label == Label::FeedrateChangeOnly || label == Label::Home || label == Label::MysteryMove
        {
            continue;
        }
        let radius = (flow / std::f32::consts::PI).sqrt();
        let length = start.distance(end);
        let direction = end - start;
        let mut sphere = false;

        // Create the mesh and material
        let mesh_handle = match label {
            Label::PlanarExtrustion | Label::NonPlanarExtrusion | Label::PrePrintMove => meshes
                .add(Cylinder {
                    radius,
                    half_height: length / 2.0,
                }),
            Label::TravelMove | Label::LiftZ | Label::LowerZ | Label::Wipe => {
                meshes.add(Cylinder {
                    radius: 0.1,
                    half_height: length / 2.0,
                })
            }
            Label::DeRetraction | Label::Retraction => {
                sphere = true;
                meshes.add(Sphere {
                    radius: radius * 1.618,
                })
            }
            _ => panic!(),
        };
        let material_handle = match label {
            Label::PlanarExtrustion | Label::NonPlanarExtrusion | Label::PrePrintMove => materials
                .add(StandardMaterial {
                    base_color: settings.extrusion_color,
                    ..Default::default()
                }),
            Label::TravelMove | Label::LiftZ | Label::LowerZ | Label::Wipe => {
                materials.add(StandardMaterial {
                    base_color: settings.travel_color,
                    ..Default::default()
                })
            }
            Label::DeRetraction => materials.add(StandardMaterial {
                base_color: settings.deretraction_color,
                ..Default::default()
            }),
            Label::Retraction => materials.add(StandardMaterial {
                base_color: settings.retraction_color,
                ..Default::default()
            }),
            _ => panic!(),
        };

        // Calculate the middle point and orientation of the cylinder
        let middle = (start + end) / 2.0;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
        let translation = if sphere { end } else { middle };
        let e_id = commands
            .spawn((
                PbrBundle {
                    mesh: mesh_handle,
                    material: material_handle,
                    transform: Transform {
                        translation,
                        rotation,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PickableBundle::default(),
                Tag { id },
            ))
            .id();
        map.0.insert(id, e_id);
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

pub fn match_objects(
    mut p_query: Query<(&mut PickingInteraction, Entity, &Tag)>,
    id_map: Res<IdMap>,
) {
    let mut p_map: std::collections::HashMap<Tag, PickingInteraction> =
        std::collections::HashMap::new();
    let ids = id_map.0.values().collect::<HashSet<_>>();
    for (p, e, t) in p_query.iter_mut() {
        if ids.contains(&e) {
            p_map.insert(*t, *p);
        }
    }
    for (mut p, _, t) in p_query.iter_mut() {
        *p = *p_map.get(t).unwrap();
    }
}
