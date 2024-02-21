#![windows_subsystem = "windows"]
use bevy::{
    ecs::query,
    prelude::*,
    render::view::window,
    sprite::collide_aabb::{collide, Collision},
    window::{PresentMode, WindowTheme},
};
mod space_invaders;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Rgba {
            red: 0.,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        }))
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Invaders".into(),
                        resolution: space_invaders::WINDOW_SIZE.into(),
                        present_mode: PresentMode::AutoVsync,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        enabled_buttons: bevy::window::EnabledButtons {
                            maximize: false,
                            ..Default::default()
                        },
                        // This will spawn an invisible window
                        // The window will be made visible in the make_visible() system after 3 frames.
                        // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                        visible: true,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
            space_invaders::SpaceInvaders,
        ))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Camera;

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>, mut window: Query<&mut Window>) {
    let mut cam = Camera2dBundle {
        projection: OrthographicProjection {
            // don't forget to set `near` and `far`
            near: -1000.0,
            far: 1000.0,
            // ... any other settings you want to change ...
            ..default()
        },
        ..default()
    };

    cam.projection.scale = 1.;
    cmd.spawn((cam, Camera));
}
