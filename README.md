# af_bevy_specs

This crate provides a way to specify your game objects in .ron formatted 'spec' files.

## Boilerplate

See the `spec_test.rs` example.

Include the spec plugin:

```rust
    app
        .add_plugins(spec::SpecPlugins)
```

Register your spec types:

```rust
        .register_spec::<GunSpec>("gun.spec")
```

Load the specs from a folder inside `assets`

```rust
        .add_spec_folder("specs")
```

## Spec Type Definition

Define your Spec types:

```rust
#[derive(Deserialize, Debug, TypeUuid, Clone, Spec)]
#[uuid = "5c027c98-3984-469b-83f0-d7d910ff02b9"]
pub struct GunSpec {
    pub name: String,
    pub bullet: NamedHandle<BulletSpec>,
}
```

Use the `NamedHandle` type for values that are other specs or any asset that can be loaded with `AssetServer.load()`

## Spec Files

```ron
GunSpec (
    name: "Bright Blue Nerf Gun",
    bullet: "specs/nerf_bullet.bullet.spec",
)
```

## In Systems

In your systems, specs appear as assets

```rust
fn example_startup_system(gun_specs: Res<Assets<GunSpec>>) { ... 
```

`Assets<>` is augmented with `get_named` and `get_named_expect` methods to help you navigate around spec structures.

## Loading

Currently spec af_bevy_specs has an `AppState::Loading` phase during which all specs are loaded from disk. After this phase, all specs have their handles populated. If you want to further process specs after this, use the following:

```rust
        app.add_system_set(
            SystemSet::on_exit(AppState::Loading)
                .with_system(finalize_sprite_sheet_spec)
                .after("populate"),
        );
```
