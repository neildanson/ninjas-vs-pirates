use std::time::Duration;

use bevy::{
    audio::{PlaybackMode, Volume, VolumeLevel},
    prelude::*,
    window::{close_on_esc, WindowMode},
};
use bevy_hanabi::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

const LEFT_KEY: KeyCode = KeyCode::A;
const RIGHT_KEY: KeyCode = KeyCode::D;
const PUNCH_KEY: KeyCode = KeyCode::P;
const KICK_KEY: KeyCode = KeyCode::K;
const RUN_FORWARD_SPEED: f32 = 4.0;
const RUN_BACKWARDS_SPEED: f32 = -2.5;

const HANDS_COLLISION_GROUP: u32 = 1;
const FEET_COLLISION_GROUP: u32 = 2;
const BODY_COLLISION_GROUP: u32 = 4;

#[derive(Default, PartialEq, Copy, Clone, Debug)]
enum AnimationState {
    #[default]
    Idle,
    Punching,
    Kicking,
    Running,
    RunningBackwards,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Cameraman;
#[derive(Component, Default)]
struct CharacterState {
    player_state: AnimationState,
    old_player_state: AnimationState,
    current_animation_timer: Option<Timer>,
}

impl CharacterState {
    fn update_player_state(&mut self, new_state: AnimationState) {
        self.old_player_state = self.player_state;
        self.player_state = new_state;
        if self.old_player_state != self.player_state {
            //println!("Old Player state: {:?}", self.old_player_state);
            //println!("Player state: {:?}", self.player_state);
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
        transform: Transform::from_xyz(0.0, 3.0, 12.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
        ..default()
    };

    commands.spawn(camera).insert(Cameraman);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2500.0,
            shadows_enabled: true,
            shadow_depth_bias : 0.001,
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

fn setup_ninja(mut commands: Commands, asset_server: Res<AssetServer>) {
    let guy = asset_server.load("ninja.glb#Scene0");

    commands
        .spawn(SceneBundle {
            scene: guy.clone_weak(),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0)).with_translation(Vec3::new(-3.0,0.0,0.0)),
            ..default()
        })
        .insert(Player)
        .insert(CharacterState::default());

    commands.insert_resource(Animations {
        idle: asset_server.load("ninja.glb#Animation0"),
        kick: asset_server.load("ninja.glb#Animation1"),
        punch: asset_server.load("ninja.glb#Animation2"),
        run_forwards: asset_server.load("ninja.glb#Animation3"),
        walk_backwards: asset_server.load("ninja.glb#Animation4"),
    });
}

fn setup_pirate(mut commands: Commands, asset_server: Res<AssetServer>) {
    let guy = asset_server.load("pirate.glb#Scene0");

    commands.spawn(SceneBundle {
        scene: guy.clone_weak(),
        transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI / 2.0)).with_translation(Vec3::new(3.0,0.0,0.0)).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        ..default()
    })
    .insert(Enemy)
    .insert(CharacterState::default());
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut animation_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut animation_player in &mut animation_players.iter_mut() {
        animation_player.play(animations.idle.clone_weak()).repeat();
    }
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

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

fn process_input(keys: Res<Input<KeyCode>>, time: Res<Time>, mut players: Query<&mut CharacterState, With<Player>>) {
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
            } else {
                continue;
            }
        }
        let mut new_state = AnimationState::Idle;
        if keys.just_pressed(PUNCH_KEY) {
            new_state = AnimationState::Punching;
        } else if keys.just_pressed(KICK_KEY) {
            new_state = AnimationState::Kicking;
        } else if keys.pressed(RIGHT_KEY) && !keys.pressed(LEFT_KEY) {
            new_state = AnimationState::Running;
        } else if keys.pressed(LEFT_KEY) && !keys.pressed(RIGHT_KEY) {
            new_state = AnimationState::RunningBackwards;
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
    mut character_state: Query<&mut CharacterState>,
) {
    let transition_duration = Duration::from_secs_f32(0.2);
    for (parent, mut animation_player) in animation_players.iter_mut() {
        //Should make this a function
        let parent_entity = parent_query.get(parent.get()).unwrap();
        let character_state = character_state.get_mut(parent_entity.get());
        match character_state {
            Ok(mut character_state) => {
                if character_state.player_state == character_state.old_player_state
                    || character_state.current_animation_timer.is_some()
                {
                    continue;
                }

                match character_state.player_state {
                    AnimationState::Idle => {
                        animation_player
                            .play_with_transition(animations.idle.clone(), transition_duration)
                            .repeat();
                    }
                    AnimationState::Punching => {
                        animation_player
                            .play_with_transition(animations.punch.clone(), transition_duration)
                            .set_speed(1.5);
                        character_state.current_animation_timer =
                            Some(Timer::from_seconds(0.6, TimerMode::Once));
                        commands.spawn(AudioBundle {
                            source: asset_server.load("punch.ogg"),
                            settings: PlaybackSettings {
                                mode: PlaybackMode::Despawn,
                                volume: Volume::Relative(VolumeLevel::new(0.4)),
                                ..Default::default()
                            },
                            ..default()
                        });
                    }
                    AnimationState::Kicking => {
                        animation_player
                            .play_with_transition(animations.kick.clone(), transition_duration)
                            .set_speed(1.5);
                        character_state.current_animation_timer =
                            Some(Timer::from_seconds(1.0, TimerMode::Once));
                        commands.spawn(AudioBundle {
                            source: asset_server.load("kick.ogg"),
                            settings: PlaybackSettings {
                                mode: PlaybackMode::Despawn,
                                volume: Volume::Relative(VolumeLevel::new(0.4)),
                                ..Default::default()
                            },
                            ..default()
                        });
                    }
                    AnimationState::Running => {
                        animation_player
                            .play_with_transition(
                                animations.run_forwards.clone(),
                                transition_duration,
                            )
                            .repeat();
                    }
                    AnimationState::RunningBackwards => {
                        animation_player
                            .play_with_transition(
                                animations.walk_backwards.clone(),
                                transition_duration,
                            )
                            .repeat();
                    }
                }
            }
            _ => {}
        }
    }
}

fn process_movement(time: Res<Time>, mut player: Query<(&mut Transform, &CharacterState)>) {
    for (mut controller, player) in player.iter_mut() {
        if player.player_state == AnimationState::Running {
            controller.translation += Vec3::new(RUN_FORWARD_SPEED * time.delta_seconds(), 0.0, 0.0);
        } else if player.player_state == AnimationState::RunningBackwards {
            controller.translation +=
                Vec3::new(RUN_BACKWARDS_SPEED * time.delta_seconds(), 0.0, 0.0);
        }
        controller.translation.x = controller.translation.x.clamp(-4.0, 4.0);
    }
}

fn add_collision_point(
    commands: &mut Commands,
    entity: Entity,
    collision_group: u32,
    debug_color: Color,
    radius: f32,
) {
    commands
        .entity(entity)
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(radius))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ColliderDebugColor(debug_color))
        .insert(CollisionGroups::new(
            Group::from_bits_truncate(collision_group),
            Group::from_bits_truncate(collision_group),
        ))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC);
}

fn calculate_collision_points<T:Component>(
    mut is_run: Local<bool>,
    mut commands: Commands,
    players: Query<Entity, With<T>>,
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
                if name.as_str().starts_with("hand") {
                    add_collision_point(
                        &mut commands,
                        entity,
                        HANDS_COLLISION_GROUP,
                        Color::BLUE,
                        0.15,
                    );
                }

                if name.as_str().starts_with("foot") {
                    add_collision_point(
                        &mut commands,
                        entity,
                        FEET_COLLISION_GROUP,
                        Color::BLUE,
                        0.15,
                    );
                }

                if name.as_str().starts_with("spine_02") {
                    add_collision_point(
                        &mut commands,
                        entity,
                        BODY_COLLISION_GROUP,
                        Color::RED,
                        0.4,
                    );
                }
            }
        }
    }
}

fn display_events(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    //mut effects: ResMut<Assets<EffectAsset>>,
    mut collision_events: EventReader<CollisionEvent>,
    names: Query<&Name>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                if let Some(contact_pair) = rapier_context.contact_pair(*entity1, *entity2) {
                    //let name1 = names.get(*entity1).unwrap();
                    //let name2 = names.get(*entity2).unwrap();

                    //println!("Collision started: {:?} {:?}", name1, name2);
                    //for manifold in contact_pair.manifolds() {
                    //    for solver_contact in manifold.solver_contacts() {
                    //        spawn_particles(&mut commands, &mut effects, solver_contact.point());
                    //    }
                    //}
                    //println!("Received collision event: {:?}", collision_event);
                }
            }
            _ => {}
        }
    }
}

/*
fn spawn_particles(
    commands: &mut Commands,
    effects: &mut ResMut<Assets<EffectAsset>>,
    position: Vec3,
) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.2));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.2, Vec2::splat(0.01));
    size_gradient1.add_key(0.2, Vec2::splat(0.1));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);


    let init_pos = SetPositionSphereModifier {
        center: writer.lit(position).expr(),
        radius: writer.lit(0.2).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(2.0)).expr(),
    };

    let effect = EffectAsset::new(
        2048,
        Spawner::once(250.0.into(), true),
        writer.finish(),
    )
    .with_name("firework")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .render(ColorOverLifetimeModifier {
        gradient: color_gradient1,
    })
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient1,
        screen_space_size: false,
    });

    let effect1 = effects.add(effect);

    /*commands.spawn((
        Name::new("firework"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
    ));*/
}
*/

fn update_cameraman(
    ninja: Query<&Transform, (With<Player>, Without<Enemy>, Without<Cameraman>)>,
    pirate: Query<&Transform, (With<Enemy>, Without<Player>, Without<Cameraman>)>,
    mut cameraman: Query<&mut Transform, (With<Cameraman>, Without<Enemy>, Without<Player>)>,
) {
    let ninja = ninja.single();
    let pirate = pirate.single();
    let mut cameraman = cameraman.single_mut();
    let look_at = (ninja.translation + pirate.translation) / 2.0;
    cameraman.look_at(look_at, Vec3::Y);
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
        .add_plugins(WorldInspectorPlugin::new()) //If debug
        .add_plugins(HanabiPlugin) //If debug
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_ninja,
                setup_pirate,
                setup_background,
                setup_music,
            ),
        )
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                process_input,
                process_animation,
                process_movement,
                calculate_collision_points::<Player>,
                calculate_collision_points::<Enemy>,
                display_events,
                update_cameraman,
            ),
        )
        .add_systems(Update, close_on_esc)
        .run();
}
