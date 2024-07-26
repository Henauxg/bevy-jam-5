use bevy::{
    gltf::GltfMesh,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};

use super::spawn::dummy::DummyCachedData;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HandleMap<ImageKey>>();
    app.init_resource::<HandleMap<ImageKey>>();

    app.register_type::<HandleMap<SfxKey>>();
    app.init_resource::<HandleMap<SfxKey>>();

    app.register_type::<HandleMap<SoundtrackKey>>();
    app.init_resource::<HandleMap<SoundtrackKey>>();

    app.register_type::<HandleMap<SceneKey>>();
    app.init_resource::<HandleMap<SceneKey>>();

    app.register_type::<HandleMap<GltfKey>>();
    app.init_resource::<HandleMap<GltfKey>>();

    app.register_type::<HandleMap<AnimationKey>>();
    app.init_resource::<HandleMap<AnimationKey>>();
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ImageKey {
    Ducky,
}

impl AssetKey for ImageKey {
    type Asset = Image;
}

impl FromWorld for HandleMap<ImageKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [(
            ImageKey::Ducky,
            asset_server.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        )]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SfxKey {
    ButtonHover,
    ButtonPress,
    Step1,
    Step2,
    Step3,
    Step4,
}

impl AssetKey for SfxKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SfxKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SfxKey::ButtonHover,
                asset_server.load("audio/sfx/button_hover.ogg"),
            ),
            (
                SfxKey::ButtonPress,
                asset_server.load("audio/sfx/button_press.ogg"),
            ),
            (SfxKey::Step1, asset_server.load("audio/sfx/step1.ogg")),
            (SfxKey::Step2, asset_server.load("audio/sfx/step2.ogg")),
            (SfxKey::Step3, asset_server.load("audio/sfx/step3.ogg")),
            (SfxKey::Step4, asset_server.load("audio/sfx/step4.ogg")),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SoundtrackKey {
    Credits,
    Gameplay,
    Excavation,
}

impl AssetKey for SoundtrackKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SoundtrackKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SoundtrackKey::Credits,
                asset_server.load("audio/soundtracks/Monkeys Spinning Monkeys.ogg"),
            ),
            (
                SoundtrackKey::Gameplay,
                asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
            ),
            (
                SoundtrackKey::Excavation,
                asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
            ),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SceneKey {
    Rock,
    Gladiator,
    Sword,
    Dummy,
    Arena,
    GroundDetails,
}

impl AssetKey for SceneKey {
    type Asset = Scene;
}

impl FromWorld for HandleMap<SceneKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (SceneKey::Rock, asset_server.load("models/rock.glb#Scene0")),
            (
                SceneKey::Gladiator,
                asset_server.load("models/gladiator.glb#Scene0"),
            ),
            (
                SceneKey::Dummy,
                asset_server.load("models/dummy.glb#Scene0"),
            ),
            (
                SceneKey::Sword,
                asset_server.load("models/sword.glb#Scene0"),
            ),
            (
                SceneKey::Arena,
                asset_server.load("models/arena.glb#Scene0"),
            ),
            (
                SceneKey::GroundDetails,
                asset_server.load("models/ground.glb#Scene0"),
            ),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AnimationKey {
    GladiatorWalk,
    GladiatorIdle,
    GladiatorSlash,
}

impl AssetKey for AnimationKey {
    type Asset = AnimationClip;
}

impl FromWorld for HandleMap<AnimationKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                AnimationKey::GladiatorWalk,
                asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/gladiator.glb")),
            ),
            // Slash too ?
            // (
            //     AnimationKey::GladiatorIdle,
            //     asset_server.load(GltfAssetLabel::Animation(2).from_asset("models/gladiator.glb")),
            // ),
            (
                AnimationKey::GladiatorSlash,
                asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/gladiator.glb")),
            ),
            (
                AnimationKey::GladiatorIdle,
                asset_server.load(GltfAssetLabel::Animation(4).from_asset("models/gladiator.glb")),
            ),
        ]
        .into()
    }
}

pub const ASSETS_SCALE: f32 = 0.015;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum GltfKey {
    Rock,
    Gladiator,
    Dummy,
}

impl AssetKey for GltfKey {
    type Asset = Gltf;
}

impl FromWorld for HandleMap<GltfKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (GltfKey::Rock, asset_server.load("models/rock.glb")),
            (
                GltfKey::Gladiator,
                asset_server.load("models/gladiator.glb"),
            ),
            (GltfKey::Dummy, asset_server.load("models/dummy.glb")),
        ]
        .into()
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);

impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<K: AssetKey> HandleMap<K> {
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.values()
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }
}

#[derive(Resource, Default)]
/// Flags tracking which assets still need to be processed
pub struct AssetsProcessing {
    pub dummy: bool,
}

pub fn process_dummy_asset(
    mut commands: Commands,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    meshes: ResMut<Assets<Mesh>>,
    mut assets_processing: ResMut<AssetsProcessing>,
) {
    let gltf_handle = &gltf_handles[&GltfKey::Dummy];
    let Some(gltf) = assets_gltf.get(gltf_handle) else {
        return;
    };
    let Some(gltf_mesh) = assets_gltfmesh.get(&gltf.meshes[0]) else {
        return;
    };
    let mesh_handle = &gltf_mesh.primitives[0].mesh;
    let Some(mesh) = meshes.get(mesh_handle) else {
        return;
    };

    let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::ConvexHull) else {
        return;
    };
    commands.insert_resource(DummyCachedData { collider });
    assets_processing.dummy = true;
}

pub fn all_assets_loaded(
    asset_server: Res<AssetServer>,
    image_handles: Res<HandleMap<ImageKey>>,
    sfx_handles: Res<HandleMap<SfxKey>>,
    soundtrack_handles: Res<HandleMap<SoundtrackKey>>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    scene_handles: Res<HandleMap<SceneKey>>,
    animation_handles: Res<HandleMap<AnimationKey>>,
) -> bool {
    image_handles.all_loaded(&asset_server)
        && sfx_handles.all_loaded(&asset_server)
        && soundtrack_handles.all_loaded(&asset_server)
        && gltf_handles.all_loaded(&asset_server)
        && scene_handles.all_loaded(&asset_server)
        && animation_handles.all_loaded(&asset_server)
}

pub fn all_assets_processed(assets_processing: Res<AssetsProcessing>) -> bool {
    assets_processing.dummy
}
