use std::{marker::PhantomData, sync::Arc};

use bevy::{
    asset::{Asset, LoadState},
    prelude::*,
    reflect::TypeUuid,
};

use bevy_common_assets::ron::RonAssetPlugin;

pub struct SpecPlugin;

impl Plugin for SpecPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(AppState::Loading)
            .insert_resource(SpecFoldersToLoad::default())
            .add_system_set(
                SystemSet::on_enter(AppState::Loading).with_system(load_specs_startup_system),
            )
            .insert_resource(LoadingHandles::default())
            .insert_resource(AllLoaded::default())
            .add_system_set(SystemSet::on_update(AppState::Loading).with_system(monitor_spec_load));
    }
}

fn monitor_spec_load(
    ass: ResMut<AssetServer>,
    loading_handles: Res<LoadingHandles>,
    mut all_loaded: ResMut<AllLoaded>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut loaded = 0;
    let mut loading = 0;
    let mut other = 0;
    for lh in &loading_handles.0 {
        let load_state = ass.get_load_state(lh.clone());
        match load_state {
            LoadState::Loaded => loaded += 1,
            LoadState::Loading => loading += 1,
            _ => other += 1,
        }
    }
    if loaded > 0 && loading == 0 && other == 0 && !all_loaded.0 {
        all_loaded.0 = true;
        app_state.set(AppState::Playing).unwrap();
    }
}

#[derive(Resource, Default)]
struct AllLoaded(bool);

fn populate_handles<Spec>(mut specs: ResMut<Assets<Spec>>, mut ass: ResMut<AssetServer>)
where
    Spec: Asset + PopulateHandles,
{
    for (_handle_id, spec) in specs.iter_mut() {
        spec.populate_handles(&mut ass);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Loading,
    Playing,
}

pub trait AppHelperSpecRegisterTrait {
    fn register_spec<Spec: PopulateHandles>(&mut self, extension_name: &'static str) -> &mut App
    where
        for<'de> Spec: serde::Deserialize<'de> + Asset;
}

impl AppHelperSpecRegisterTrait for App {
    fn register_spec<Spec: PopulateHandles>(&mut self, extension_name: &'static str) -> &mut App
    where
        for<'de> Spec: serde::Deserialize<'de> + Asset,
    {
        let app = self
            .insert_resource(SpecLoadingHandles::<Spec>::new())
            .add_plugin(RonAssetPlugin::<Spec>::new(&[extension_name]))
            .add_system_set(
                SystemSet::on_exit(AppState::Loading)
                    .with_system(populate_handles::<Spec>)
                    .label("populate"),
            );
        app
    }
}

#[derive(Resource, Default)]
struct SpecFoldersToLoad {
    folders: Vec<String>,
}

pub trait AppHelperSpecLoaderTrait {
    fn add_spec_folder(&mut self, folder_name: &str) -> &mut App;
}

impl AppHelperSpecLoaderTrait for App {
    fn add_spec_folder(&mut self, folder_name: &str) -> &mut App {
        let mut sftl = self
            .world
            .get_resource_mut::<SpecFoldersToLoad>()
            .expect("SpecFoldersToLoad resource not found. Is the SpecPlugin added?");
        sftl.folders.push(String::from(folder_name));
        self
    }
}

#[derive(Resource)]
pub struct SpecLoadingHandles<SpecType> {
    pub handles: Vec<HandleUntyped>,
    _p: PhantomData<SpecType>,
}

impl<SpecType> SpecLoadingHandles<SpecType> {
    pub fn new() -> Self {
        Self {
            handles: vec![],
            _p: PhantomData,
        }
    }
}

#[derive(Resource, Default)]
pub struct LoadingHandles(pub Vec<HandleUntyped>);

#[derive(Debug, Clone)]
pub struct NamedHandle<T: TypeUuid + Send + Sync + 'static> {
    pub name: String,
    pub maybe_handle: Option<Arc<Handle<T>>>,
}

impl<T: TypeUuid + Send + Sync + 'static> NamedHandle<T> {
    pub fn get_handle(&self) -> Handle<T> {
        return (self.maybe_handle.clone().unwrap().clone())
            .as_ref()
            .clone();
    }
}

impl<'de, T: TypeUuid + Send + Sync> serde::Deserialize<'de> for NamedHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(NamedHandle::<T> {
            name: s,
            maybe_handle: None,
        })
    }
}

impl<T: TypeUuid + Send + Sync + 'static> NamedHandle<T> {
    pub fn load(&self, ass: &AssetServer) -> Handle<T> {
        ass.load(&self.name)
    }
}

pub trait GetNamedTrait<T: Asset> {
    fn get_named(&self, h: &NamedHandle<T>) -> Option<&T>;
    fn get_named_expect(&self, h: &NamedHandle<T>) -> &T;
}

impl<T: Asset> GetNamedTrait<T> for Assets<T> {
    fn get_named(&self, h: &NamedHandle<T>) -> Option<&T> {
        let h = &h
            .maybe_handle
            .clone()
            .expect(format!("get_named failed for {} - handle does not exist", h.name).as_str());
        self.get(h)
    }
    fn get_named_expect(&self, h: &NamedHandle<T>) -> &T {
        return self
            .get_named(h)
            .expect(format!("get_named failed for {} - asset does not exist", h.name).as_str());
    }
}

pub trait PopulateHandles {
    fn populate_handles(&mut self, ass: &mut AssetServer);
}

fn load_specs_startup_system(
    ass: Res<AssetServer>,
    mut loading_handles: ResMut<LoadingHandles>,
    folders: Res<SpecFoldersToLoad>,
) {
    for folder in folders.folders.iter() {
        loading_handles
            .0
            .append(&mut ass.load_folder(folder).unwrap());
    }
}
