use std::path::PathBuf;

use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

use bevy_renderware::dff::Dff;
use clap::Parser;

pub mod maps;
use maps::ipl_parser::Ipl;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// If provided, render this asset by itself
    #[clap(short, long)]
    path: Option<PathBuf>,
}

struct DesiredAssetRenderPath(PathBuf);
struct DesiredAssetMeshes(Vec<(Handle<Dff>, Transform, bool)>);

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_renderware::RwPlugin)
        .add_plugin(maps::ipl_parser::IplPlugin)
        .add_plugin(EditorPlugin)
        .insert_resource(DesiredAssetMeshes(vec![]));

    if let Some(path) = args.path {
        app.insert_resource(DesiredAssetRenderPath(path.strip_prefix("assets/")?.into()))
            .add_startup_system(asset_viewer);
    } else {
        app.add_startup_system(load_maps);
    };

    app.add_system(process_pending_desired_meshes)
        .add_system(handle_ipl_events)
        .run();

    Ok(())
}

fn asset_viewer(
    mut commands: Commands,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    asset_server: Res<AssetServer>,
    desired_asset_render_path: Res<DesiredAssetRenderPath>,
) {
    desired_asset_meshes.0.push((
        asset_server.load(desired_asset_render_path.0.as_path()),
        Transform::from_xyz(0.0, 0.5, 0.0),
        false,
    ));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-1.0, 1.0, -1.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        ..default()
    });
}

fn process_pending_desired_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    asset_server: Res<AssetServer>,
    asset_meshes: Res<Assets<Dff>>,
) {
    for (handle, transform, spawned) in &mut desired_asset_meshes.0 {
        if *spawned {
            continue;
        }

        if let Some(dff) = asset_meshes.get(handle.clone()) {
            let mesh = meshes.add(dff.mesh.clone());
            let dff_material = dff.materials.get(0);
            let asset_path = dff.asset_paths.get(0);

            let base_color = dff_material
                .map(|m| Color::rgba_u8(m.color.r, m.color.g, m.color.b, m.color.a))
                .unwrap_or(Color::WHITE);
            let base_color_texture = asset_path.map(|ap| asset_server.load(ap.clone()));

            commands.spawn_bundle(PbrBundle {
                mesh,
                material: materials.add(StandardMaterial {
                    base_color,
                    base_color_texture,
                    unlit: true,
                    ..default()
                }),
                transform: transform.clone(),
                ..default()
            });
            *spawned = true;
        }
    }
}

fn load_maps(mut commands: Commands, asset_server: Res<AssetServer>) {
    const MAP_PATHS: &[&str] = &[
        "data/maps/airport/airport.ipl",
        "data/maps/airportN/airportN.ipl",
        "data/maps/bank/bank.ipl",
        "data/maps/bridge/bridge.ipl",
        "data/maps/cisland/cisland.ipl",
        "data/maps/club/CLUB.ipl",
        "data/maps/concerth/concerth.ipl",
        // "data/maps/cull.ipl",
        "data/maps/docks/docks.ipl",
        "data/maps/downtown/downtown.ipl",
        "data/maps/downtows/downtows.ipl",
        "data/maps/golf/golf.ipl",
        "data/maps/haiti/haiti.ipl",
        "data/maps/haitiN/haitin.ipl",
        "data/maps/hotel/hotel.ipl",
        // "data/maps/islandsf/islandsf.ipl",
        "data/maps/lawyers/lawyers.ipl",
        "data/maps/littleha/littleha.ipl",
        "data/maps/mall/mall.ipl",
        "data/maps/mansion/mansion.ipl",
        "data/maps/nbeach/nbeach.ipl",
        "data/maps/nbeachbt/nbeachbt.ipl",
        "data/maps/nbeachw/nbeachw.ipl",
        "data/maps/oceandn/oceandN.ipl",
        "data/maps/oceandrv/oceandrv.ipl",
        // "data/maps/paths.ipl",
        "data/maps/starisl/starisl.ipl",
        "data/maps/stripclb/stripclb.ipl",
        "data/maps/washintn/washintn.ipl",
        "data/maps/washints/washints.ipl",
        "data/maps/yacht/yacht.ipl",
    ];

    commands.insert_resource(
        MAP_PATHS
            .iter()
            .map(|path| asset_server.load(*path))
            .collect::<Vec<Handle<Ipl>>>(),
    );

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::ONE * 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn handle_ipl_events(
    mut ev_asset: EventReader<AssetEvent<Ipl>>,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Ipl>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let ipl = assets.get(handle).unwrap();

                for (name, [x, y, z]) in &ipl.instances {
                    if name.len() > 3 && name[..3].eq_ignore_ascii_case("lod") {
                        continue;
                    }

                    let path = format!("models/gta3/{name}.dff");
                    let model_handle = asset_server.load(&path);

                    desired_asset_meshes.0.push((
                        model_handle,
                        Transform::from_xyz(*x, *y, *z),
                        false,
                    ));
                }
            }
            AssetEvent::Modified { handle: _handle } => {
                panic!("you aren't meant to modify the IPLs during gameplay!");
            }
            AssetEvent::Removed { handle: _handle } => {}
        }
    }
}
