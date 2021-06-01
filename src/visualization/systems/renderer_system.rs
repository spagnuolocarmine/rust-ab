use bevy::prelude::{ColorMaterial, Handle, Query, Res, Transform, Visible};

use crate::bevy::prelude::Commands;
use crate::visualization::agent_render::{AgentRender, SpriteType};
use crate::visualization::asset_handle_factory::AssetHandleFactoryResource;
use crate::visualization::wrappers::{ActiveSchedule, ActiveState};

/// The system that updates the visual representation of each agent of our simulation.
pub fn renderer_system<A: AgentRender + Clone>(
    mut query: Query<(
        &mut A,
        &mut Transform,
        &mut Visible,
        &mut Handle<ColorMaterial>,
    )>,
    state_wrapper: Res<ActiveState<A>>,
    schedule_wrapper: Res<ActiveSchedule<A>>,
    mut sprite_factory: AssetHandleFactoryResource,
    mut commands: Commands,
) {
    for (mut render, mut transform, mut visible, mut material) in query.iter_mut() {
        render.update(&mut *transform, &state_wrapper.0, &mut *visible);
        let SpriteType::Emoji(emoji_code) = render.sprite();
        let new_material = sprite_factory.get_material_handle(emoji_code);
        if *material != new_material {
            *material = new_material;
        }
    }
    for new_agent in &schedule_wrapper.0.newly_scheduled {
        let SpriteType::Emoji(emoji_code) = new_agent.sprite();
        let sprite_render = sprite_factory.get_emoji_loader(emoji_code);
        new_agent
            .clone()
            .setup_graphics(sprite_render, &mut commands, &state_wrapper.0);
    }
}
