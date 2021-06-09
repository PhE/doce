use bevy::prelude::*;

use crate::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(button_interactions.system());
    }
}

fn button_interactions(
    ui_resources: Res<UIResources>,
    mut button_query: Query<
        (&Interaction, &Children, &mut Handle<ColorMaterial>),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children, mut material) in button_query.iter_mut() {
        match interaction {
            Interaction::None => {
                *material = ui_resources.white.clone();

                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.sections[0].style.color = Color::BLACK;
                    }
                }
            }
            Interaction::Hovered => {
                *material = ui_resources.black.clone();

                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.sections[0].style.color = Color::WHITE;
                    }
                }
            }
            _ => (),
        }
    }
}
