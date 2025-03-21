use std::time::Duration;

use bevy::{input::{keyboard::{Key, KeyboardInput}, ButtonState}, prelude::*, window::PrimaryWindow};
use iyes_cli::prelude::*;
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_clicommand_args("hello", hello_world)
        .register_clicommand_noargs("help", show_help)
        .register_clicommand_noargs("spawn", spawn_sprite_random)
        .register_clicommand_args("spawn", spawn_sprite_at)
        .register_clicommand_noargs("despawn", despawn_sprites)
        .add_systems(Startup, (setup, setup_console))
        .add_systems(Update, (mouseclicks, console_text_input, despawn_timeout))
        .run();
}

/// Implementation of the "hello" command
fn hello_world(In(args): In<Vec<String>>) {
    print!("Hello");
    for arg in args {
        print!(", {}", arg);
    }
    println!("!");
}

/// Implementation of the "spawn" command (noargs variant)
fn spawn_sprite_random(q_window: Query<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    let window = q_window.single().unwrap();
    let mut rng = thread_rng();
    commands.spawn((
        DespawnTimeout(Timer::new(Duration::from_secs(5), TimerMode::Once)),
        Sprite {
            color: Color::srgb(0.9, 0.1, 0.7),
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        Transform::from_xyz(
            rng.gen_range(0.0..window.width()),
            rng.gen_range(0.0..window.height()),
            1.0,
        ),
    ));
}

/// Implementation of the "spawn" command (args variant)
fn spawn_sprite_at(In(args): In<Vec<String>>, mut commands: Commands) {
    if args.len() != 2 {
        error!("spawn command must take exactly 2 args!");
        return;
    }
    let Ok(x) = args[0].parse::<f32>() else {
        error!("spawn command args must be numbers!");
        return;
    };
    let Ok(y) = args[1].parse::<f32>() else {
        error!("spawn command args must be numbers!");
        return;
    };

    commands.spawn((
        DespawnTimeout(Timer::new(Duration::from_secs(5), TimerMode::Once)),
        Sprite {
            color: Color::srgb(0.9, 0.1, 0.7),
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        Transform::from_xyz(x, y, 1.0),
    ));
}

/// Implementation of the "despawn" command
fn despawn_sprites(mut commands: Commands, q: Query<Entity, With<Sprite>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn setup(world: &mut World) {
    // Example: you can call clicommands from exclusive systems
    world.run_cli("hello");
    world.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            viewport_origin: Vec2::ZERO,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Example: you can call clicommands from regular systems, using Commands
fn mouseclicks(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = q_window.single().unwrap();
        if let Some(cursor) = window.cursor_position() {
            commands.run_cli(&format!(
                "spawn {} {}",
                cursor.x,
                window.height() - cursor.y
            ));
        }
    }
    if mouse.just_pressed(MouseButton::Middle) {
        commands.run_cli("spawn");
    }
    if mouse.just_pressed(MouseButton::Right) {
        commands.run_cli("despawn");
    }
}

#[derive(Component)]
struct CliPrompt;

/// Implement a simple "console" to type commands in
fn console_text_input(
    mut commands: Commands,
    mut evr_kbd: EventReader<KeyboardInput>,
    mut text: Single<&mut Text, With<CliPrompt>>,
) {
    for ev in evr_kbd.read() {
        if let ButtonState::Released = ev.state {
            continue;
        }
        match (&ev.key_code, &ev.logical_key) {
            (KeyCode::Escape, _) => {
                text.0 = String::new();
            }
            (KeyCode::Enter, _) => {
                commands.run_cli(&text.0);
                text.0 = String::new();
            }
            (KeyCode::Backspace, _) => {
                text.0.pop();
            }
            (_, Key::Space) => {
                text.0.push(' ');
            }
            (_, Key::Character(s)) => {
                text.0.push_str(s.as_str());
            }
            _ => {}
        }
    }
}

fn setup_console(world: &mut World) {
    let font = world.resource::<AssetServer>().load("Ubuntu-R.ttf");
    let console = world
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(5.0),
                left: Val::Percent(5.0),
                top: Val::Auto,
                right: Val::Auto,
                padding: UiRect::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.9, 0.8, 0.7)),
        ))
        .id();
    let prompt = world
        .spawn((
            CliPrompt,
            Text::new(""),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::BLACK),
        ))
        .id();
    world.entity_mut(console).add_children(&[prompt]);
}

#[derive(Component)]
struct DespawnTimeout(Timer);

fn show_help(world: &mut World) {
    let font = world.resource::<AssetServer>().load("Ubuntu-R.ttf");
    let help_box = world
        .spawn((
            DespawnTimeout(Timer::new(Duration::from_secs(5), TimerMode::Once)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(5.0),
                left: Val::Percent(5.0),
                bottom: Val::Auto,
                right: Val::Auto,
                padding: UiRect::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgb(0.9, 0.8, 0.7)),
        ))
        .id();
    let text = world
        .spawn((
            Text::new(
                "Available console commands: \"help\", \"hello\", \"spawn\", \"spawn <x> <y>\", \"despawn\".\n
                Left/Right mouse click will run \"spawn\"/\"despawn\".",
            ),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
        ))
        .id();
    world.entity_mut(help_box).add_children(&[text]);
}

fn despawn_timeout(
    mut commands: Commands,
    t: Res<Time>,
    mut q: Query<(Entity, &mut DespawnTimeout)>,
) {
    for (e, mut timeout) in &mut q {
        timeout.0.tick(t.delta());
        if timeout.0.finished() {
            commands.entity(e).despawn();
        }
    }
}
