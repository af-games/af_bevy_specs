use bevy::prelude::*;
use spec::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(spec::SpecPlugins)
        .register_spec::<GunSpec>("gun.spec")
        .register_spec::<BulletSpec>("bullet.spec")
        .add_spec_folder("specs")
        .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(example_startup_system))
        .run();
}

#[derive(Deserialize, Debug, TypeUuid, Clone, Spec)]
#[uuid = "5c027c98-3984-469b-83f0-d7d910ff02b9"]
pub struct GunSpec {
    pub name: String,
    pub bullet: NamedHandle<BulletSpec>,
}

#[derive(Deserialize, Debug, TypeUuid, Clone, Spec)]
#[uuid = "ec667baf-cebb-4fd4-a42f-25e8fede3f9b"]
pub struct BulletSpec {
    pub damage: f32,
    pub name: String,
    pub image: NamedHandle<Image>,
}

fn example_startup_system(
    mut commands: Commands,
    gun_specs: Res<Assets<GunSpec>>,
    bullet_specs: Res<Assets<BulletSpec>>,
) {
    println!("Example Startup System...");
    commands.spawn(Camera2dBundle::default());
    for (_, gs) in gun_specs.iter() {
        println!("Got a gun spec:");
        dbg!(gs);
        println!("Getting bullet spec from gun spec...");
        let bs = bullet_specs.get_named_expect(&gs.bullet);
        println!("Got a bullet spec:");
        dbg!(bs);
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(32.0)),
                ..Default::default()
            },
            texture: bs.image.get_handle(),
            ..Default::default()
        });
    }
}
