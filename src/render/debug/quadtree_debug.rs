use bevy::prelude::*;
use crate::sim::SimulatorExt as _;
use super::*;

#[derive(Resource)]
pub struct QuadtreeDebugState {
    view_level: usize,
    max_level: usize,
}

#[derive(Component)]
pub struct QuadTreeItem {
    level: usize
}

pub fn setup_quadtree(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    nbody: Res<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    use sim::QuadTreeUtils;

    println!("Setup Quadtree Debug");

    let Some(tree) = nbody.get_quadtree() else { return; };
    let max_vis_depth = tree.depth.min(8);

    commands.insert_resource(QuadtreeDebugState {
        view_level: 0,
        max_level: max_vis_depth
    });

    let circle = &render_data.circle_mesh;
    let mat = &render_data.force_material;
    let square = &render_data.square_mesh;
    let red = &render_data.red_material;
    let yellow = &render_data.yellow_material;
    for depth in 0 ..= max_vis_depth {
        let cells = tree.layer_at_depth(depth);

        let cell_size = QuadTreeUtils::layer_width(depth);
        for (idx, cell) in cells.iter().enumerate() {
            let cell_pos =
                tree.norm.invert_normalize(
                    QuadTreeUtils::position_at_layer(depth, idx)
                    + sim::Vec2f::new(QuadTreeUtils::layer_width(depth + 1), QuadTreeUtils::layer_width(depth + 1))
                );

            let cell_size = tree.norm.invert_normalize_size(
                sim::Vec2f::new(cell_size, cell_size)
            );

            let text_size = (2.0 / (depth + 1) as f32).min(1.0);

            commands.spawn((
                Visibility::Visible,
                Transform::from_xyz(cell_pos.x, cell_pos.y, 1.0),
                QuadTreeItem { level: depth },
                SimEntityTag,
            ))
            .with_child((
                Mesh2d(circle.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_xyz(cell.position.x - cell_pos.x, cell.position.y - cell_pos.y, 1.0)
                    .with_scale([cell.mass.sqrt(), cell.mass.sqrt(), 1.0].into()),
            ))
            .with_child((
                Mesh2d(square.to_owned()),
                if cell.mass == 0.0 {
                    MeshMaterial2d(yellow.to_owned())
                } else {
                    MeshMaterial2d(red.to_owned())
                },
                Transform::from_scale(Vec3::from([cell_size.x - 1.0, cell_size.y - 1.0, 1.0])),
            ))
            .with_child((
                Text2d::new(format!("{}", idx)),
                Transform::from_scale(Vec3::from([text_size, text_size, 1.0]))
            ));
        }
    }
}

pub fn process_inputs_debug_quadtree(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    debug_data: Option<ResMut<QuadtreeDebugState>>,
    quadtree_items: Query<(&mut Visibility, &QuadTreeItem)>,
) {
    let Some(mut debug_data) = debug_data else { return; };

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowUp) {
        if debug_data.view_level < debug_data.max_level {
            debug_data.view_level += 1;
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowDown) {
        if debug_data.view_level > 0 {
            debug_data.view_level -= 1;
        }
    }

    for (mut vis, item) in quadtree_items {
        if item.level == debug_data.view_level {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn destroy_quadtree(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<QuadTreeItem>>
) {
    commands.remove_resource::<QuadtreeDebugState>();

    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}

