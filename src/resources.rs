use bevy::prelude::*;

use crate::*;

pub struct ShaderResources {
    pub checkerboard_material: Handle<CheckerboardMaterial>,
    pub checkerboard_render_pipelines: RenderPipelines,
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
        app.add_startup_system(init_shader_resources.system())
            .add_startup_system(init_ui_resources.system());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "c16c38f6-53fe-499c-832f-acc879f36454"]
pub struct CheckerboardMaterial {
    pub first_color: Color,
    pub second_color: Color,
}

fn init_shader_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<CheckerboardMaterial>>,
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

    let checkerboard_material = materials.add(CheckerboardMaterial {
        first_color: Color::GRAY,
        second_color: Color::WHITE,
    });

    commands.insert_resource(ShaderResources {
        checkerboard_material,
        checkerboard_render_pipelines,
    });
}

fn init_ui_resources(mut commands: Commands, asset_server: Res<AssetServer>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(UIResources {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        transparent: materials.add(Color::NONE.into()),
        white: materials.add(Color::WHITE.into()),
        black: materials.add(Color::BLACK.into()),
    });
}
