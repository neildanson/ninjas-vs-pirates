use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;



#[derive(Component)]
struct Player;

#[derive(Resource)]
struct Animations {
    idle: Handle<AnimationClip>,
    walk_forwards: Handle<AnimationClip>,
    walk_backwards: Handle<AnimationClip>,
    punch: Handle<AnimationClip>,
    kick : Handle<AnimationClip>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));

    let camera = Camera3dBundle {
        camera: Camera {
            ..default()
        },
        transform : Transform::from_xyz(0.0, 3.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    let guy = asset_server.load("ninja.glb#Scene0");
    commands
        .spawn(SceneBundle {
            scene: guy.clone_weak(),
            ..default()
        })
        .insert(Player)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(1.0, 2.0, 1.0))
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(KinematicCharacterController {
            // The character offset is set to 0.01.
            offset: CharacterLength::Absolute(0.01),
            ..default()
        });
        ;
    commands.insert_resource(Animations {
        idle: asset_server.load("ninja.glb#Animation0"),
        kick: asset_server.load("ninja.glb#Animation1"),
        punch: asset_server.load("ninja.glb#Animation2"),
        walk_forwards: asset_server.load("ninja.glb#Animation2"),
        walk_backwards: asset_server.load("ninja.glb#Animation3"),
    });
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        println!("setup_scene_once_loaded");
        player.play(animations.idle.clone_weak()).repeat();
    }
}


fn setup_background(
    mut commands: Commands, asset_server: Res<AssetServer>,
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
    commands
        .spawn(SceneBundle {
            scene: background.clone_weak(),
            transform : Transform::from_scale(Vec3::ONE * 5.0),
            ..default()
        });
}

fn process_input(
    keys: Res<Input<KeyCode>>,
    animations: Res<Animations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    parent_query: Query<&Parent>,//, With<Player>>,
    mut player: Query<(&mut KinematicCharacterController, &mut Transform)>,//, With<Player>>,
) {
    
    for (parent, mut animation_player) in animation_players.iter_mut() {
        let transition_duration = Duration::from_secs_f32(0.1);
        if keys.just_pressed(KeyCode::P) {
            animation_player
                .play_with_transition(animations.punch.clone(), transition_duration);
        }

        if keys.just_pressed(KeyCode::K) {
            animation_player
                .play_with_transition(animations.kick.clone(), transition_duration);
        }
        /*
        if keys.pressed(KeyCode::W) && keys.just_pressed(KeyCode::ShiftLeft) {
            animation_player
                .play_with_transition(animations.run.clone(), transition_duration)
                .repeat();
        }

        if keys.just_pressed(KeyCode::W) && !keys.pressed(KeyCode::ShiftLeft) {
            animation_player
                .play_with_transition(animations.walk.clone(), transition_duration)
                .repeat();
        }

        if keys.pressed(KeyCode::W) && keys.just_released(KeyCode::ShiftLeft) {
            animation_player
                .play_with_transition(animations.walk.clone(), transition_duration)
                .repeat();
        }

        if keys.just_released(KeyCode::W) {
            animation_player
                .play_with_transition(animations.idle.clone(), transition_duration)
                .repeat();
        }
        //Should make this a function
        let parent_entity = parent_query.get(parent.get()).unwrap();
        let mut player  = player.get_mut(parent_entity.get()).unwrap();
        

        if keys.just_pressed(KeyCode::A) {
            animation_player
                .play_with_transition(animations.left_turn.clone(), transition_duration);
        }

        if keys.pressed(KeyCode::A) {
            player.1.rotate_local_y(0.005);// *= Quat::from_rotation_y(0.005);
        }

        if keys.pressed(KeyCode::W) {
            if keys.pressed(KeyCode::ShiftLeft) {
                player.0.translation = Some(Vec3::new(0.0, 0.0, 0.02));
            } else {
                player.0.translation = Some(Vec3::new(0.0, 0.0, 0.01));
            }
        }*/
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
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Update, (setup_scene_once_loaded, process_input))
        .add_systems(Startup, (setup, setup_background))
        //.add_system(controls)
        .run();
}
