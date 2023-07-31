use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};
use iyes_cli::prelude::*;
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_clicommand_noargs("hello", hello_world)
        .register_clicommand_noargs("help", show_help)
        .register_clicommand_noargs("spawn", spawn_sprite_random)
        .register_clicommand_args("spawn", spawn_sprite_at)
        .register_clicommand_noargs("despawn", despawn_sprites)
        .add_systems(Startup, (setup, setup_console))
        .add_systems(Update, (mouseclicks, console_text_input, despawn_timeout))
        .run();
}

/// Implementation of the "hello" command
fn hello_world() {
    println!("Hello, World!");
}

/// Implementation of the "spawn" command (noargs variant)
fn spawn_sprite_random(q_window: Query<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    let window = q_window.single();
    let mut rng = thread_rng();
    commands.spawn((
        DespawnTimeout(Timer::new(Duration::from_secs(5), TimerMode::Once)),
        SpriteBundle {
            sprite: Sprite {
                color: Color::PINK,
                custom_size: Some(Vec2::splat(64.0)),
                ..default()
            },
            transform: Transform::from_xyz(
                rng.gen_range(0.0..window.width()),
                rng.gen_range(0.0..window.height()),
                1.0,
            ),
            ..default()
        },
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
        SpriteBundle {
            sprite: Sprite {
                color: Color::PINK,
                custom_size: Some(Vec2::splat(64.0)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 1.0),
            ..default()
        },
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
    world.run_clicommand("hello");
    let mut camera = Camera2dBundle::default();
    camera.projection.viewport_origin = Vec2::ZERO;
    world.spawn(camera);
}

/// Example: you can call clicommands from regular systems, using Commands
/// (they will run on `apply_system_buffers`)
fn mouseclicks(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = q_window.single();
        if let Some(cursor) = window.cursor_position() {
            commands.run_clicommand(&format!(
                "spawn {} {}",
                cursor.x,
                window.height() - cursor.y
            ));
        }
    }
    if mouse.just_pressed(MouseButton::Middle) {
        commands.run_clicommand("spawn");
    }
    if mouse.just_pressed(MouseButton::Right) {
        commands.run_clicommand("despawn");
    }
}

#[derive(Component)]
struct CliPrompt;

/// Implement a simple "console" to type commands in
fn console_text_input(
    mut commands: Commands,
    mut evr_char: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    mut query: Query<&mut Text, With<CliPrompt>>,
) {
    if kbd.just_pressed(KeyCode::Escape) {
        for mut text in &mut query {
            text.sections[1].value = String::new();
        }
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Return) {
        for mut text in &mut query {
            commands.run_clicommand(&text.sections[1].value);
            text.sections[1].value = String::new();
        }
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Back) {
        for mut text in &mut query {
            text.sections[1].value.pop();
        }
        evr_char.clear();
        return;
    }
    for ev in evr_char.iter() {
        for mut text in &mut query {
            text.sections[1].value.push(ev.char);
        }
    }
}

fn setup_console(world: &mut World) {
    let font = world.resource::<AssetServer>().load("Ubuntu-R.ttf");
    let console = world
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(5.0),
                left: Val::Percent(5.0),
                top: Val::Auto,
                right: Val::Auto,
                padding: UiRect::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BEIGE),
            ..Default::default()
        })
        .id();
    let prompt_style = TextStyle {
        font: font.clone(),
        font_size: 24.0,
        color: Color::RED,
    };
    let input_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        color: Color::BLACK,
    };
    let prompt = world
        .spawn((
            CliPrompt,
            TextBundle {
                text: Text::from_sections([
                    TextSection::new("~ ", prompt_style),
                    TextSection::new("", input_style),
                ]),
                ..Default::default()
            },
        ))
        .id();
    world.entity_mut(console).push_children(&[prompt]);
}

#[derive(Component)]
struct DespawnTimeout(Timer);

fn show_help(world: &mut World) {
    let font = world.resource::<AssetServer>().load("Ubuntu-R.ttf");
    let help_box = world
        .spawn((
            DespawnTimeout(Timer::new(Duration::from_secs(5), TimerMode::Once)),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(5.0),
                    left: Val::Percent(5.0),
                    bottom: Val::Auto,
                    right: Val::Auto,
                    padding: UiRect::all(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::BEIGE),
                ..Default::default()
            },
        ))
        .id();
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 12.0,
        color: Color::BLACK,
    };
    let prompt = world
        .spawn((TextBundle {
            text: Text::from_section(
                "Available console commands: \"help\", \"hello\", \"spawn\", \"spawn <x> <y>\", \"despawn\".\n
                Left/Right mouse click will run \"spawn\"/\"despawn\".",
                text_style,
            ),
            ..Default::default()
        },))
        .id();
    world.entity_mut(help_box).push_children(&[prompt]);
}

fn despawn_timeout(
    mut commands: Commands,
    t: Res<Time>,
    mut q: Query<(Entity, &mut DespawnTimeout)>,
) {
    for (e, mut timeout) in &mut q {
        timeout.0.tick(t.delta());
        if timeout.0.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}
