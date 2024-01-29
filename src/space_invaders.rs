use bevy::{prelude::*, ui::SetUiViewBindGroup};

pub const WINDOW_SIZE: Vec2 = Vec2::new(900., 600.);
// delta x: 450 | delta y: 300

pub const ALIEN_TOP: f32 = 250.;
pub const ALIEN_BOTTOM: f32 = 0.;
pub const ALIEN_LEFT: f32 = -200.;
pub const ALIEN_RIGHT: f32 = 200.;

pub const PADDING_ALIEN: f32 = 4.;
pub const SIZE_ALIEN: Vec2 = Vec2::new(28., 20.);

pub const DEATH_LINE: f32 = -100.;
pub const TOP_END: f32 = 300.;

pub struct SpaceInvaders;

impl Plugin for SpaceInvaders {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
pub struct Tank;

#[derive(Component)]
pub struct Alien;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Barrier;

#[derive(Component)]
pub struct AlienGroup;

#[derive(Component)]
pub struct KillBullets;

#[derive(Component)]
pub struct DeathLine;

#[derive(Component)]
pub struct Lives(u8);

#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct Alive(bool);

#[derive(Component)]
pub struct Position(Vec2);

/*
    - Place all Aliens in a Box for easier movement (relative coords to parent)
    -
*/

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    // Bricks
    let total_width_of_aliens = ALIEN_RIGHT - ALIEN_LEFT;
    let total_height_of_aliens = ALIEN_TOP - ALIEN_BOTTOM;

    assert!(total_width_of_aliens > 0.0);
    assert!(total_height_of_aliens > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_aliens / (SIZE_ALIEN.x + PADDING_ALIEN * 2.)).floor() as usize;
    let n_rows = (total_height_of_aliens / (SIZE_ALIEN.y + PADDING_ALIEN * 2.)).floor() as usize;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_aliens = (ALIEN_LEFT + ALIEN_RIGHT) / 2.0;
    let left_edge_of_aliens = center_of_aliens
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * (SIZE_ALIEN.x + PADDING_ALIEN * 2.));

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_aliens + SIZE_ALIEN.x / 2.;
    let offset_y = ALIEN_BOTTOM + SIZE_ALIEN.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (SIZE_ALIEN.x + 2. * PADDING_ALIEN),
                offset_y + row as f32 * (SIZE_ALIEN.y + 2. * PADDING_ALIEN),
            );

            // brick
            cmd.spawn((
                SpriteBundle {
                    texture: asset_server.load("alien2x.png"),
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                Alien,
                // Collider,
            ));
        }
    }
}
