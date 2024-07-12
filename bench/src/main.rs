//! This example provides a 2D benchmark.
//!
//! It was hastily copied from Bevy's `bevymark`, and could probably
//! use some cleanup

use bevy::{
    color::palettes::basic::*,
    core::FrameCount,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    utils::Duration,
    window::{PresentMode, WindowResolution},
    winit::{UpdateMode, WinitSettings},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const GRAVITY: f32 = -9.8 * 100.0;
const MAX_VELOCITY: f32 = 750.;
const BIRD_SCALE: f32 = 0.15;
const BIRD_TEXTURE_SIZE: usize = 256;
const HALF_BIRD_SIZE: f32 = BIRD_TEXTURE_SIZE as f32 * BIRD_SCALE * 0.5;
const MAX_BIRDS: usize = 100000;
const BIRDS_PER_WAVE: usize = 1000;

#[derive(Resource)]
struct BevyCounter {
    pub count: usize,
    pub color: Color,
}

#[derive(Component)]
struct Bird {
    velocity: Vec3,
}

const FIXED_TIMESTEP: f32 = 0.2;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "BevyMark".into(),
                    resolution: WindowResolution::new(1280.0, 720.0)
                        .with_scale_factor_override(1.0),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .insert_resource(BevyCounter {
            count: 0,
            color: Color::WHITE,
        })
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, scheduled_spawner)
        .add_systems(
            Update,
            (movement_system, collision_system, counter_system, measure),
        )
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
            FIXED_TIMESTEP,
        )))
        .run();
}

fn scheduled_spawner(
    mut commands: Commands,
    windows: Query<&Window>,
    mut counter: ResMut<BevyCounter>,
    bird_resources: ResMut<BirdResources>,
) {
    if counter.count >= MAX_BIRDS {
        return;
    }

    let window = windows.single();

    let bird_resources = bird_resources.into_inner();
    spawn_birds(
        &mut commands,
        &window.resolution,
        &mut counter,
        BIRDS_PER_WAVE,
        bird_resources,
        None,
    );
}

#[derive(Resource)]
struct BirdResources {
    texture: Handle<Image>,
    color_rng: ChaCha8Rng,
    velocity_rng: ChaCha8Rng,
    transform_rng: ChaCha8Rng,
}

#[derive(Component)]
struct StatsText;

#[allow(clippy::too_many_arguments)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bird_resources = BirdResources {
        // We're seeding the PRNG here to make this example deterministic for testing purposes.
        // This isn't strictly required in practical use unless you need your app to be deterministic.
        texture: asset_server.load("icon.png"),
        color_rng: ChaCha8Rng::seed_from_u64(42),
        velocity_rng: ChaCha8Rng::seed_from_u64(42),
        transform_rng: ChaCha8Rng::seed_from_u64(42),
    };

    let text_section = move |color: Srgba, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font_size: 40.0,
                color: color.into(),
                ..default()
            },
        )
    };

    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            z_index: ZIndex::Global(i32::MAX),
            background_color: Color::BLACK.with_alpha(0.75).into(),
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                TextBundle::from_sections([
                    text_section(LIME, "Bird Count: "),
                    text_section(AQUA, ""),
                    text_section(LIME, "\nFPS (raw): "),
                    text_section(AQUA, ""),
                    text_section(LIME, "\nFPS (avg): "),
                    text_section(AQUA, ""),
                ]),
                StatsText,
            ));
        });

    commands.insert_resource(bird_resources);
}

fn bird_velocity_transform(
    half_extents: Vec2,
    mut translation: Vec3,
    velocity_rng: &mut ChaCha8Rng,
    waves: Option<usize>,
    dt: f32,
) -> (Transform, Vec3) {
    let mut velocity = Vec3::new(MAX_VELOCITY * (velocity_rng.gen::<f32>() - 0.5), 0., 0.);

    if let Some(waves) = waves {
        // Step the movement and handle collisions as if the wave had been spawned at fixed time intervals
        // and with dt-spaced frames of simulation
        for _ in 0..(waves * (FIXED_TIMESTEP / dt).round() as usize) {
            step_movement(&mut translation, &mut velocity, dt);
            handle_collision(half_extents, &translation, &mut velocity);
        }
    }
    (
        Transform::from_translation(translation).with_scale(Vec3::splat(BIRD_SCALE)),
        velocity,
    )
}

const FIXED_DELTA_TIME: f32 = 1.0 / 60.0;

#[allow(clippy::too_many_arguments)]
fn spawn_birds(
    commands: &mut Commands,
    primary_window_resolution: &WindowResolution,
    counter: &mut BevyCounter,
    spawn_count: usize,
    bird_resources: &mut BirdResources,
    waves_to_simulate: Option<usize>,
) {
    let bird_x = (primary_window_resolution.width() / -2.) + HALF_BIRD_SIZE;
    let bird_y = (primary_window_resolution.height() / 2.) - HALF_BIRD_SIZE;

    let half_extents = 0.5 * primary_window_resolution.size();

    let batch = (0..spawn_count)
        .map(|_| {
            let bird_z = bird_resources.transform_rng.gen::<f32>();

            let (transform, velocity) = bird_velocity_transform(
                half_extents,
                Vec3::new(bird_x, bird_y, bird_z),
                &mut bird_resources.velocity_rng,
                waves_to_simulate,
                FIXED_DELTA_TIME,
            );

            let color = Color::linear_rgb(
                bird_resources.color_rng.gen(),
                bird_resources.color_rng.gen(),
                bird_resources.color_rng.gen(),
            );

            (
                SpriteBundle {
                    transform,
                    texture: bird_resources.texture.clone(),
                    sprite: Sprite { color, ..default() },
                    ..default()
                },
                Bird { velocity },
            )
        })
        .collect::<Vec<_>>();
    commands.spawn_batch(batch);

    counter.count += spawn_count;
    counter.color = Color::linear_rgb(
        bird_resources.color_rng.gen(),
        bird_resources.color_rng.gen(),
        bird_resources.color_rng.gen(),
    );
}

fn step_movement(translation: &mut Vec3, velocity: &mut Vec3, dt: f32) {
    translation.x += velocity.x * dt;
    translation.y += velocity.y * dt;
    velocity.y += GRAVITY * dt;
}

fn movement_system(mut bird_query: Query<(&mut Bird, &mut Transform)>) {
    let dt = FIXED_DELTA_TIME;

    for (mut bird, mut transform) in &mut bird_query {
        step_movement(&mut transform.translation, &mut bird.velocity, dt);
    }
}

fn handle_collision(half_extents: Vec2, translation: &Vec3, velocity: &mut Vec3) {
    if (velocity.x > 0. && translation.x + HALF_BIRD_SIZE > half_extents.x)
        || (velocity.x <= 0. && translation.x - HALF_BIRD_SIZE < -half_extents.x)
    {
        velocity.x = -velocity.x;
    }
    let velocity_y = velocity.y;
    if velocity_y < 0. && translation.y - HALF_BIRD_SIZE < -half_extents.y {
        velocity.y = -velocity_y;
    }
    if translation.y + HALF_BIRD_SIZE > half_extents.y && velocity_y > 0.0 {
        velocity.y = 0.0;
    }
}
fn collision_system(windows: Query<&Window>, mut bird_query: Query<(&mut Bird, &Transform)>) {
    let window = windows.single();

    let half_extents = 0.5 * window.size();

    for (mut bird, transform) in &mut bird_query {
        handle_collision(half_extents, &transform.translation, &mut bird.velocity);
    }
}

fn counter_system(
    counter: Res<BevyCounter>,
    mut query: Query<&mut Text, With<StatsText>>,
    time: Res<Time>,
    frame_count: Res<FrameCount>,
) {
    let mut text = query.single_mut();

    if counter.is_changed() {
        text.sections[1].value = counter.count.to_string();
    }

    text.sections[3].value = format!("{:.2}", 1. / time.delta_seconds());

    text.sections[5].value = format!("{:.2}", frame_count.0 as f32 / time.elapsed_seconds());
}

fn measure(
    counter: ResMut<BevyCounter>,
    time: Res<Time>,
    frame_count: Res<FrameCount>,
    mut start_time: Local<Option<f32>>,
    mut start_frame: Local<Option<u32>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    if counter.count < MAX_BIRDS {
        return;
    }

    if start_time.is_none() {
        info!("Starting measurement");
        *start_time = Some(time.elapsed_seconds());
        *start_frame = Some(frame_count.0);
        return;
    }

    let start_time = start_time.unwrap();
    let start_frame = start_frame.unwrap();

    let elapsed = time.elapsed_seconds() - start_time;

    if elapsed >= 5. {
        *done = true;
        info!(
            "Average Frame Time: {:.2}ms",
            elapsed / (frame_count.0 as f32 - start_frame as f32) * 1000.0
        );
    }
}
