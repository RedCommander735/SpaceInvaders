use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2d};

pub const WINDOW_SIZE: Vec2 = Vec2::new(900., 600.);
// delta x: 450 | delta y: 300

pub const ALIEN_TOP: f32 = 250.;
pub const ALIEN_BOTTOM: f32 = 0.;
pub const ALIEN_LEFT: f32 = -200.;
pub const ALIEN_RIGHT: f32 = 200.;

pub const PADDING_ALIEN: f32 = 4.;
pub const ALIEN_SIZE: Vec2 = Vec2::new(28., 20.);

const VERT_STEP: f32 = ALIEN_SIZE.y + 2. * PADDING_ALIEN;

const ALIEN_SPEED: f32 = 10.;
const VERT_STEP_DISTANCE_FROM_MIDDLE: u32 = 50;

pub const DEATH_LINE: f32 = -100.;
pub const TOP_END: f32 = 300.;

pub struct SpaceInvaders;

impl Plugin for SpaceInvaders {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            // .add_systems(FixedUpdate, (move_aliens, check_for_collisions).chain())
            .add_event::<CollisionEvent>();
    }
}

#[derive(Event, Default)]
pub struct CollisionEvent;

#[derive(Component)]
struct Collider;

#[derive(Component)]
pub struct Cannon;

#[derive(Component)]
pub struct Alien;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Barrier;

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
struct Movement {
    speed: f32,
    direction: Direction,
}

#[derive(Component)]
struct HorizontalStep(f32);

#[derive(Clone, Copy)]
enum Direction {
    LEFT = -1,
    RIGHT = 1,
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    cmd.spawn(());

    cmd.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::from_xyz(0., DEATH_LINE, 0.).with_scale(Vec3 {
                x: WINDOW_SIZE.x,
                y: 1.,
                z: 0.,
            }),
            material: materials.add(ColorMaterial::from(Color::RED)),
            visibility: Visibility::Visible,
            ..default()
        },
        DeathLine,
        Collider,
    ));

    spawn_aliens(&mut cmd, 4, 7, &asset_server);

    // Spawn Connon
    // cmd.spawn((
    //     SpriteBundle {
    //         texture: asset_server.load("alien2x.png"),
    //         transform: Transform::from_xyz(0., DEATH_LINE - 20., 0.)
    //         ..default()
    //     },
    //     Alien,
    //     Collider,
    //     Alive(true););)

    cmd.spawn((
        SpriteBundle {
            texture: asset_server.load("tank2x.png"),
            transform: Transform::from_xyz(0., DEATH_LINE - 50., 0.),
            ..Default::default()
        },
        Cannon,
        Collider,
        Alive(true),
        Lives(3),
    ));

    cmd.spawn(ColorMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3 {
            x: 6.,
            y: 6.,
            z: 0.,
        }),
        material: materials.add(ColorMaterial::from(Color::RED)),
        visibility: Visibility::Visible,
        ..default()
    });
}

fn check_for_collisions(
    mut commands: Commands,
    // mut scoreboard: ResMut<Scoreboard>,
    mut alien_query: Query<&Transform, With<Alien>>,
    collider_query: Query<(Entity, &Transform, Option<&DeathLine>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    // check collision with walls
    for (collider_entity, transform, maybe_death_line) in &collider_query {
        for alien in &mut alien_query {
            let collision = collide(
                alien.translation,
                ALIEN_SIZE,
                transform.translation,
                transform.scale.truncate(),
            );
            if let Some(collision) = collision {
                // Sends a collision event so that other systems can react to the collision
                collision_events.send_default();

                // Bricks should be despawned and increment the scoreboard on collision
                if maybe_death_line.is_some() {
                    // TODO - Implement life loss and round restart here
                    todo!()
                }
            }
        }
    }
}

fn spawn_aliens(
    cmd: &mut Commands,
    n_rows: u32,    // y
    n_columns: u32, // x
    asset_server: &Res<AssetServer>,
) {
    // let alien_bottom_edge = (n_rows as f32 * (ALIEN_SIZE.y + PADDING_ALIEN * 2.));
    // Add variable to change height depending on level
    let alien_bottom_edge = DEATH_LINE + 100.;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_aliens = 0.;
    let left_edge_of_aliens = center_of_aliens
        - (n_columns as f32 / 2.0 * (ALIEN_SIZE.x + PADDING_ALIEN * 2.))
        + PADDING_ALIEN;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_aliens + ALIEN_SIZE.x / 2.;
    let offset_y = alien_bottom_edge + ALIEN_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (ALIEN_SIZE.x + 2. * PADDING_ALIEN),
                offset_y + row as f32 * (ALIEN_SIZE.y + 2. * PADDING_ALIEN),
            );

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
                Collider,
                Alive(true),
                Movement {
                    speed: ALIEN_SPEED,
                    direction: Direction::RIGHT,
                },
                HorizontalStep(0.),
            ));
        }
    }
}

fn move_aliens(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Movement, &mut HorizontalStep, &mut Transform), With<Alien>>,
) {
    for (mut movement, mut h_step, mut transform) in &mut query {
        let delta = movement.speed * movement.direction as i8 as f32 * time.delta_seconds();
        transform.translation.x += delta;
        h_step.0 += delta;
        // eprintln!("{}", h_step.0);

        if h_step.0.abs() >= VERT_STEP_DISTANCE_FROM_MIDDLE as f32 {
            transform.translation.y -= VERT_STEP;
            movement.direction = match movement.direction {
                Direction::LEFT => Direction::RIGHT,
                Direction::RIGHT => Direction::LEFT,
            }
        }
    }
}
