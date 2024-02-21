use std::time::Duration;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2d};
use bevy::time::Stopwatch;

pub const WINDOW_SIZE: Vec2 = Vec2::new(900., 600.);
// delta x: 450 | delta y: 300

const PADDING_ALIEN: f32 = 4.;
const ALIEN_SIZE: Vec2 = Vec2::new(28., 20.);
const TANK_SIZE: Vec2 = Vec2::new(24., 28.);
const BULLET_SIZE: Vec2 = Vec2::new(4., 12.);

const VERT_STEP: f32 = ALIEN_SIZE.y + 2. * PADDING_ALIEN;

const TANK_SPEED: f32 = 50.;
const BULLET_SPEED: f32 = 100.;

const VERT_STEP_DISTANCE_FROM_MIDDLE: u32 = 50;

const BULLET_LIMIT: i16 = 5;

const DEATH_LINE: f32 = -100.;

const SCOREBOARD_FONT_SIZE: f32 = 20.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const SCORE_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

pub struct SpaceInvaders;

impl Plugin for SpaceInvaders {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .insert_resource(TimeSinceLastShot {
                time: Stopwatch::new(),
            })
            .insert_resource(Scoreboard { score: 0 })
            .insert_resource(Level { level: 1 })
            .insert_resource(CurrentSize {
                n_rows: 4,
                n_columns: 7,
            })
            .insert_resource(AlienSpeed { speed: 10. })
            .insert_resource(Running {
                running: true
            })
            .add_event::<CollisionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    move_aliens,
                    move_tank,
                    fire_bullet,
                    bullet_system,
                    check_for_collisions,
                )
                    .chain(),
            )
            .add_systems(Update, (update_text, check_win));
    }
}

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Tank;

#[derive(Component)]
struct Alien;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct KillBullets;

#[derive(Component)]
struct DeathLine;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct ScoreBoard;

#[derive(Resource)]
struct TimeSinceLastShot {
    time: Stopwatch,
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Resource)]
struct Level {
    level: usize,
}

#[derive(Resource)]
struct CurrentSize {
    n_rows: u32,
    n_columns: u32,
}

#[derive(Resource)]
struct AlienSpeed {
    speed: f32,
}

#[derive(Resource)]
struct Running {
    running: bool
}

#[derive(Component)]
struct Movement {
    direction_x: DirectionX,
    direction_y: DirectionY,
}



#[derive(Component)]
struct HorizontalStep(f32);

#[derive(Clone, Copy)]
enum DirectionX {
    NONE = 0,
    LEFT = -1,
    RIGHT = 1,
}

#[derive(Clone, Copy)]
enum DirectionY {
    NONE = 0,
    DOWN = -1,
    UP = 1,
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    size: Res<CurrentSize>,
) {
    cmd.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::from_xyz(0., DEATH_LINE, 0.).with_scale(Vec3 {
                x: WINDOW_SIZE.x,
                y: 1.,
                z: 0.,
            }),
            material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            visibility: Visibility::Visible,
            ..default()
        },
        DeathLine,
        Collider,
    ));

    spawn_aliens(&mut cmd, size.n_rows, size.n_columns, &asset_server);

    // Spawn Tank
    cmd.spawn((
        SpriteBundle {
            texture: asset_server.load("tank2x.png"),
            transform: Transform::from_xyz(0., DEATH_LINE - 50., 0.),
            ..Default::default()
        },
        Tank,
        Collider,
    ));

    // Scoreboard
    cmd.spawn(
        (TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
            TextSection::new(
                "\nLevel: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        ScoreBoard
    ));
}

fn check_for_collisions(
    mut cmd: Commands,
    // mut scoreboard: ResMut<Scoreboard>,
    mut alien_query: Query<(Entity, &Transform), With<Alien>>,
    death_line_query: Query<(Entity, &Transform), (With<Collider>, With<DeathLine>)>,
    bullet_query: Query<(Entity, &Transform), (With<Collider>, With<Bullet>)>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    mut running: ResMut<Running>
) {
    for (_collider_entity, transform) in &death_line_query {
        for (_, alien) in &alien_query {
            let collision = collide(
                alien.translation,
                ALIEN_SIZE,
                transform.translation,
                transform.scale.truncate(),
            );
            if let Some(_collision) = collision {
                collision_events.send_default();

                running.running = false;

                for (entity, _) in &alien_query {
                    cmd.entity(entity).despawn();
                }   

                cmd.spawn(
                    TextBundle::from_sections([
                        TextSection::new(
                            "GAME OVER",
                            TextStyle {
                                font_size: 40.,
                                color: Color::Rgba {
                                    red: 1.,
                                    green: 0.5,
                                    blue: 0.5,
                                    alpha: 1.,
                                },
                                ..default()
                            },
                        ),
                        TextSection::new(
                            format!("\nScore: {}", scoreboard.score),
                            TextStyle {
                                font_size: 20.,
                                color: Color::Rgba {
                                    red: 1.,
                                    green: 0.5,
                                    blue: 0.5,
                                    alpha: 1.,
                                },
                                ..default()
                            },
                        ),
                    ])
                    .with_style(Style {
                        display: Display::Flex,
                        position_type: PositionType::Absolute,
                        left: Val::Px(WINDOW_SIZE.x / 2. - 95.),
                        top: Val::Percent (40.),
                        ..default()
                    })
                    .with_text_alignment(TextAlignment::Center),
                );
            }
        }
    }

    for (collider_entity, transform) in &bullet_query {
        for (entity, alien) in &mut alien_query {
            let collision = collide(
                alien.translation,
                ALIEN_SIZE,
                transform.translation,
                BULLET_SIZE,
            );
            if let Some(_collision) = collision {
                // Sends a collision event so that other systems can react to the collision
                scoreboard.score += 10;
                collision_events.send_default();
                cmd.entity(collider_entity).despawn();
                cmd.entity(entity).despawn();
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

    let left_edge_of_aliens =
        -(n_columns as f32 / 2.0 * (ALIEN_SIZE.x + PADDING_ALIEN * 2.)) + PADDING_ALIEN;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_aliens + ALIEN_SIZE.x / 2.;
    let offset_y = alien_bottom_edge + ALIEN_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let alien_pos = Vec2::new(
                offset_x + column as f32 * (ALIEN_SIZE.x + 2. * PADDING_ALIEN),
                offset_y + row as f32 * (ALIEN_SIZE.y + 2. * PADDING_ALIEN),
            );

            cmd.spawn((
                SpriteBundle {
                    texture: asset_server.load("alien2x.png"),
                    transform: Transform {
                        translation: alien_pos.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                Alien,
                Collider,
                Movement {
                    direction_x: DirectionX::RIGHT,
                    direction_y: DirectionY::NONE,
                },
                HorizontalStep(0.),
            ));
        }
    }
}

fn move_aliens(
    mut _cmd: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Movement, &mut HorizontalStep, &mut Transform), With<Alien>>,
    speed: Res<AlienSpeed>,
) {
    for (mut movement, mut h_step, mut transform) in &mut query {
        let delta = speed.speed * movement.direction_x as i8 as f32 * time.delta_seconds();
        transform.translation.x += delta;
        h_step.0 += delta;
        // eprintln!("{}", h_step.0);

        if h_step.0.abs() >= VERT_STEP_DISTANCE_FROM_MIDDLE as f32 {
            transform.translation.y -= VERT_STEP;
            movement.direction_x = match movement.direction_x {
                DirectionX::LEFT => DirectionX::RIGHT,
                DirectionX::RIGHT => DirectionX::LEFT,
                DirectionX::NONE => DirectionX::NONE,
            }
        }
    }
}

fn move_tank(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Tank>>,
    time: Res<Time>,
) {
    let mut tank_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    let new_tank_position =
        tank_transform.translation.x + direction * TANK_SPEED * time.delta_seconds();

    let left_bound = -WINDOW_SIZE.x / 2.0 + TANK_SIZE.x / 2.0;
    let right_bound = WINDOW_SIZE.x / 2.0 - TANK_SIZE.x / 2.0;

    tank_transform.translation.x = new_tank_position.clamp(left_bound, right_bound);
}

fn fire_bullet(
    keyboard_input: Res<Input<KeyCode>>,
    mut cmd: Commands,
    query: Query<&Transform, With<Tank>>,
    bullets: Query<&Transform, With<Bullet>>,
    asset_server: Res<AssetServer>,
    mut stopwatch: ResMut<TimeSinceLastShot>,
    time: Res<Time>,
) {
    let tank_transform = query.single();

    stopwatch.time.tick(time.delta());

    // eprintln!("{:?}", stopwatch.0);

    let bullet_limit = BULLET_LIMIT - bullets.iter().count() as i16;

    if keyboard_input.just_pressed(KeyCode::Space)
        && bullet_limit > 0
        && stopwatch.time.elapsed_secs() > 0.25
    {
        stopwatch.time.reset();
        cmd.spawn((
            SpriteBundle {
                texture: asset_server.load("bullet2x.png"),
                transform: Transform {
                    translation: tank_transform.translation,
                    ..Default::default()
                },
                ..default()
            },
            Bullet,
            Collider,
            Movement {
                direction_x: DirectionX::NONE,
                direction_y: DirectionY::UP,
            },
            HorizontalStep(0.),
        ));
    }
}

fn bullet_system(
    mut bullets: Query<(&Movement, &mut Transform, Entity), With<Bullet>>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    for (movement, mut transform, entity) in &mut bullets {
        let delta = BULLET_SPEED * movement.direction_y as i8 as f32 * time.delta_seconds();
        transform.translation.y += delta;

        if transform.translation.y + BULLET_SIZE.y / 2. >= WINDOW_SIZE.y / 2. {
            cmd.entity(entity).despawn();
        }
    }
}

fn check_win(
    aliens: Query<Entity, With<Alien>>,
    mut exit: EventWriter<AppExit>,
    mut level: ResMut<Level>,
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut size: ResMut<CurrentSize>,
    mut speed: ResMut<AlienSpeed>,
    running: Res<Running>
) {
    if aliens.iter().count() == 0 && running.running {
        level.level += 1;

        let l = level.level;

        if l % 3 == 0 {
            size.n_rows += 1;
        } else if l % 3 == 1 {
            size.n_columns += 1;
        } else {
            speed.speed += 5.;
        }

        spawn_aliens(&mut cmd, size.n_rows, size.n_columns, &asset_server);
    }
}

fn update_text(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreBoard>>, level: Res<Level>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.score);
    text.sections[3].value = level.level.to_string();
}
