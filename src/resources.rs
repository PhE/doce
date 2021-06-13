use bevy::{
    pbr::AmbientLight,
    prelude::*,
    reflect::TypeUuid,
    render::renderer::RenderResources,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        shader::ShaderStages,
    },
};

pub struct PbrResources {
    pub checkerboard_material: Handle<CheckerboardMaterial>,
    pub checkerboard_render_pipelines: RenderPipelines,
    pub main_character_mesh: Handle<Mesh>,
    pub main_character_material: Handle<StandardMaterial>,
    pub weapon_mesh: Handle<Mesh>,
    pub weapon_material: Handle<StandardMaterial>,
    pub projectile_mesh: Handle<Mesh>,
    pub projectile_material: Handle<StandardMaterial>,
}

pub struct UIResources {
    pub font: Handle<Font>,
    pub transparent: Handle<ColorMaterial>,
    pub white: Handle<ColorMaterial>,
    pub black: Handle<ColorMaterial>,
}

pub struct InitResourcesPlugin;

impl Plugin for InitResourcesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Tick(0))
            .add_asset::<CheckerboardMaterial>()
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 0.1,
            })
            .insert_resource(GameReplay {
                tick: Tick(0),
                main_character_inputs: Vec::with_capacity(1024),
                main_character_inputs_index: 0,
                main_character_final_position: Vec3::ZERO,
            })
            .add_startup_system_to_stage(StartupStage::PreStartup, init_render_resources.system())
            .add_startup_system_to_stage(StartupStage::PreStartup, init_ui_resources.system());
    }
}

#[derive(Clone, Copy)]
pub struct Tick(pub usize);

pub struct GameReplay {
    pub tick: Tick,
    pub main_character_inputs: Vec<MainCharacterInput>,
    pub main_character_inputs_index: usize,
    pub main_character_final_position: Vec3,
}

pub struct MainCharacterInput {
    pub tick: Tick,
    pub movement: Vec2,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "c16c38f6-53fe-499c-832f-acc879f36454"]
pub struct CheckerboardMaterial {
    pub first_color: Color,
    pub second_color: Color,
}

fn init_render_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut checkerboard_materials: ResMut<Assets<CheckerboardMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let vertex = asset_server.load("shaders/checkerboard.vert");
    let fragment = Some(asset_server.load("shaders/checkerboard.frag"));

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex,
        fragment,
    }));
    let checkerboard_render_pipelines =
        RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline_handle)]);

    render_graph.add_system_node(
        "checkerboard_material",
        AssetRenderResourcesNode::<CheckerboardMaterial>::new(true),
    );
    render_graph
        .add_node_edge("checkerboard_material", base::node::MAIN_PASS)
        .unwrap();

    let checkerboard_material = checkerboard_materials.add(CheckerboardMaterial {
        first_color: Color::GRAY,
        second_color: Color::WHITE,
    });

    let main_character_mesh = meshes.add(Mesh::from(shape::Capsule {
        depth: 1.0,
        radius: 0.5,
        ..Default::default()
    }));

    let main_character_material = materials.add(StandardMaterial::default());

    let weapon_mesh = meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.4)));

    let weapon_material = materials.add(Color::BLACK.into());

    let projectile_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.1,
        depth: 0.2,
        ..Default::default()
    }));

    let projectile_material = materials.add(Color::YELLOW.into());

    commands.insert_resource(PbrResources {
        checkerboard_material,
        checkerboard_render_pipelines,
        main_character_mesh,
        main_character_material,
        weapon_mesh,
        weapon_material,
        projectile_mesh,
        projectile_material,
    });
}

fn init_ui_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(UIResources {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        transparent: materials.add(Color::NONE.into()),
        white: materials.add(Color::WHITE.into()),
        black: materials.add(Color::BLACK.into()),
    });
}
