use std::time::Duration;

use bevy::{
    audio::{PlaybackMode, Volume, VolumeLevel},
    prelude::*, window::{WindowMode, close_on_esc},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

const LEFT_KEY: KeyCode = KeyCode::A;
const RIGHT_KEY: KeyCode = KeyCode::D;
const PUNCH_KEY: KeyCode = KeyCode::P;
const KICK_KEY: KeyCode = KeyCode::K;
const RUN_FORWARD_SPEED: f32 = 4.0;
const RUN_BACKWARDS_SPEED: f32 = -2.5;

#[derive(Default, PartialEq, Copy, Clone, Debug)]
enum PlayerState {
    #[default]
    Idle,
    Punching,
    Kicking,
    Running,
    RunningBackwards,
}

#[derive(Component, Default)]
struct Player {
    current_animation_timer: Option<Timer>,
    player_state: PlayerState,
    old_player_state: PlayerState,
}

impl Player {
    fn update_player_state(&mut self, new_state: PlayerState) {
        self.old_player_state = self.player_state;
        self.player_state = new_state;
        if self.old_player_state != self.player_state {
            println!("Old Player state: {:?}", self.old_player_state);
            println!("Player state: {:?}", self.player_state);
        }
    }
}

#[derive(Resource)]
struct Animations {
    idle: Handle<AnimationClip>,
    run_forwards: Handle<AnimationClip>,
    walk_backwards: Handle<AnimationClip>,
    punch: Handle<AnimationClip>,
    kick: Handle<AnimationClip>,
}

fn setup_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.6)));

    let camera = Camera3dBundle {
        camera: Camera { ..default() },
        transform: Transform::from_xyz(0.0, 3.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    };

    commands.spawn(camera);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 5.0, -2.0),
        ..default()
    });
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let guy = asset_server.load("ninja.glb#Scene0");

    commands
        .spawn(SceneBundle {
            scene: guy.clone_weak(),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0)),
            ..default()
        })
        .insert(Player::default());

    commands.insert_resource(Animations {
        idle: asset_server.load("ninja.glb#Animation0"),
        kick: asset_server.load("ninja.glb#Animation1"),
        punch: asset_server.load("ninja.glb#Animation2"),
        run_forwards: asset_server.load("ninja.glb#Animation3"),
        walk_backwards: asset_server.load("ninja.glb#Animation4"),
    });
}

//https://bevy-cheatbook.github.io/3d/gltf.html#gltf-master-asset
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
    // mut scenes: Query<&mut Gltf, Added<Gltf>>,
) {
    for mut player in &mut players {
        println!("setup_scene_once_loaded");
        player.play(animations.idle.clone_weak()).repeat();
    }

    //for mut scene in &mut scenes {
    //    scene.update_visibility(true);
    //}
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(100.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    let background = asset_server.load("background.glb#Scene0");
    commands.spawn(SceneBundle {
        scene: background.clone_weak(),
        transform: Transform::from_scale(Vec3::ONE * 5.0),
        ..default()
    });
}

fn setup_music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("music.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        ..default()
    });

    commands.spawn(AudioBundle {
        source: asset_server.load("begin.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Relative(VolumeLevel::new(0.3)),
            ..Default::default()
        },
        ..default()
    });
}

fn process_input(keys: Res<Input<KeyCode>>, time: Res<Time>, mut players: Query<&mut Player>) {
    for mut player in players.iter_mut() {
        if player.current_animation_timer.is_some() {
            if player
                .current_animation_timer
                .as_mut()
                .unwrap()
                .tick(time.delta())
                .finished()
            {
                player.current_animation_timer = None;
                println!("Animation finished");
            } else {
                continue;
            }
        }
        let mut new_state = PlayerState::Idle;
        if keys.just_pressed(PUNCH_KEY) {
            new_state = PlayerState::Punching;
        } else if keys.just_pressed(KICK_KEY) {
            new_state = PlayerState::Kicking;
        } else if keys.pressed(RIGHT_KEY) && !keys.pressed(LEFT_KEY) {
            new_state = PlayerState::Running;
        } else if keys.pressed(LEFT_KEY) && !keys.pressed(RIGHT_KEY) {
            new_state = PlayerState::RunningBackwards;
        }
        player.update_player_state(new_state);
    }
}

fn process_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    animations: Res<Animations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    parent_query: Query<&Parent>,
    mut player: Query<&mut Player>,
) {
    let transition_duration = Duration::from_secs_f32(0.2);
    for (parent, mut animation_player) in animation_players.iter_mut() {
        //Should make this a function
        let parent_entity = parent_query.get(parent.get()).unwrap();
        let mut player = player.get_mut(parent_entity.get()).unwrap();

        if player.player_state == player.old_player_state
            || player.current_animation_timer.is_some()
        {
            continue;
        }

        match player.player_state {
            PlayerState::Idle => {
                animation_player
                    .play_with_transition(animations.idle.clone(), transition_duration)
                    .repeat();
            }
            PlayerState::Punching => {
                animation_player
                    .play_with_transition(animations.punch.clone(), transition_duration)
                    .set_speed(1.5);
                player.current_animation_timer = Some(Timer::from_seconds(0.6, TimerMode::Once));
                commands.spawn(AudioBundle {
                    source: asset_server.load("punch.ogg"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::Relative(VolumeLevel::new(0.3)),
                        ..Default::default()
                    },
                    ..default()
                });
            }
            PlayerState::Kicking => {
                animation_player
                    .play_with_transition(animations.kick.clone(), transition_duration)
                    .set_speed(1.5);
                player.current_animation_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
                commands.spawn(AudioBundle {
                    source: asset_server.load("kick.ogg"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::Relative(VolumeLevel::new(0.3)),
                        ..Default::default()
                    },
                    ..default()
                });
            }
            PlayerState::Running => {
                animation_player
                    .play_with_transition(animations.run_forwards.clone(), transition_duration)
                    .repeat();
            }
            PlayerState::RunningBackwards => {
                animation_player
                    .play_with_transition(animations.walk_backwards.clone(), transition_duration)
                    .repeat();
            }
        }
    }
}

fn process_movement(
    time: Res<Time>,
    mut player: Query<(&mut KinematicCharacterController, &Player)>,
) {
    for (mut controller, player) in player.iter_mut() {
        if player.player_state == PlayerState::Running {
            controller.translation = Some(Vec3::new(
                RUN_FORWARD_SPEED * time.delta_seconds(),
                0.0,
                0.0,
            ));
        } else if player.player_state == PlayerState::RunningBackwards {
            controller.translation = Some(Vec3::new(
                RUN_BACKWARDS_SPEED * time.delta_seconds(),
                0.0,
                0.0,
            ));
        } else {
            controller.translation = None;
        }
    }
}

fn calculate_collision_points(
    mut is_run : Local<bool>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    children: Query<&Children>,
    transforms: Query<(&Name, &Transform)>,
) {
    if *is_run {
        return;
    }
    for player in &players {
        for entity in children.iter_descendants(player) {
            if let Ok((name, _transform)) = transforms.get(entity) {
                *is_run = true;
                if name.as_str().starts_with("hand") || name.as_str().starts_with("foot") {
                    println!("Entity: {:?}", name);
                    commands
                    .entity(entity).
                        insert(RigidBody::KinematicPositionBased)
                        .insert(Collider::ball(0.2))
                        .insert(ColliderDebugColor(Color::GREEN));
                }

                if name.as_str().starts_with("eyes") {
                    println!("Entity: {:?}", name);
                    commands
                    .entity(entity).
                        insert(RigidBody::KinematicPositionBased)
                        .insert(Collider::ball(0.4))
                        .insert(ColliderDebugColor(Color::RED));
                }

                if name.as_str().starts_with("spine_02") {
                    println!("Entity: {:?}", name);
                    commands
                    .entity(entity).
                        insert(RigidBody::KinematicPositionBased)
                        .insert(Collider::ball(0.4))
                        .insert(ColliderDebugColor(Color::RED));
                }
            }
        }
    }
}

fn main() {
    App::new()
        /*/.insert_resource(WindowDescriptor {
            title: "Bob Ross".to_string(),
            width: 1024.,
            height: 512.,
            ..default()
        })*/
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(
            Startup,
            (setup_camera, setup_player, setup_background, setup_music),
        )
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                process_input,
                process_animation,
                process_movement,
                calculate_collision_points
            ),
        )
        .add_systems(Update, close_on_esc)
        .run();
}
