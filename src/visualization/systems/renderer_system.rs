use bevy::prelude::{ColorMaterial, Handle, Query, Res, Transform, Visible, World};

use crate::bevy::prelude::{Commands, ResMut};
use crate::engine::state::State;
use crate::visualization::agent_render::{AgentRender, SpriteType};
use crate::visualization::asset_handle_factory::AssetHandleFactoryResource;
use crate::visualization::simulation_descriptor::SimulationDescriptor;
use crate::visualization::visualization_state::VisualizationState;
use crate::visualization::wrappers::{ActiveSchedule, ActiveState};

/// The system that updates the visual representation of each agent of our simulation.
pub fn renderer_system<I: VisualizationState<S> + Clone + 'static, S: State>(
    mut query: Query<(
        &mut Box<dyn AgentRender>,
        &mut Transform,
        &mut Visible,
        &mut Handle<ColorMaterial>,
    )>,
    mut state_wrapper: ResMut<ActiveState<S>>,
    schedule_wrapper: Res<ActiveSchedule>,
    mut sprite_factory: AssetHandleFactoryResource,
    mut commands: Commands,
    mut vis_state: ResMut<I>,
    sim_data: Res<SimulationDescriptor>,
    //world: &mut World,
) {
    /*let mut state_wrapper = world.get_resource_mut::<ActiveState<S>>().unwrap();
    let schedule_wrapper = world.get_resource::<ActiveSchedule>().unwrap();
    let vis_state = world.get_resource::<I>().unwrap();
    let mut sprite_factory = world
        .get_resource_mut::<AssetHandleFactoryResource>()
        .unwrap();
     */
    if !sim_data.paused {
        vis_state.before_render(
            &mut state_wrapper.0,
            &schedule_wrapper.0,
            &mut commands,
            &mut sprite_factory,
        );
        let state = &Box::new(state_wrapper.0.as_state());
        for (mut agent_render, mut transform, mut visible, mut material) in query.iter_mut() {
            if let Some(agent) = vis_state.get_agent(&agent_render, state) {
                agent_render.update(&agent, &mut *transform, state, &mut *visible);
                let SpriteType::Emoji(emoji_code) = agent_render.sprite(&agent, state);
                let new_material = sprite_factory.get_material_handle(emoji_code);
                if *material != new_material {
                    *material = new_material;
                }
            }
        }
    }

    /*for (agent_impl, _) in schedule_wrapper.0.events.lock().unwrap().iter() {
        let mut agent_render = vis_state.get_agent_render(&agent_impl.agent, &mut state_wrapper.0);
       /* let mut query/*(_, transform, visible, material)*/ = world
            .query::<(
                &Box<dyn AgentRender>,
                &mut Transform,
                &mut Visible,
                &mut Handle<ColorMaterial>,
            )>();*/
        for (_, transform, visible, material) in query.iter_mut() {
            agent_render.update(
                &agent_impl.agent,
                &mut *transform,
                &Box::new(&state_wrapper.0),
                &mut *visible,
            );
            let SpriteType::Emoji(emoji_code) = agent_render.sprite(&agent_impl.agent);
            let new_material = sprite_factory.get_material_handle(emoji_code);
            if *material != new_material {
                *material = new_material;
            }
        }
    }*/

    /*for (mut render, mut transform, mut visible, mut material) in query.iter_mut() {
        let boxed_state: Box<&dyn State> = Box::new(&state_wrapper.0);
        render.update(&mut *transform, &boxed_state, &mut *visible);
        let SpriteType::Emoji(emoji_code) = render.sprite();
        let new_material = sprite_factory.get_material_handle(emoji_code);
        if *material != new_material {
            *material = new_material;
        }
    }*/
    // TODO restore
    /*for new_agent in &schedule_wrapper.0.newly_scheduled {
        let SpriteType::Emoji(emoji_code) = new_agent.sprite();
        let sprite_render = sprite_factory.get_emoji_loader(emoji_code);
        new_agent
            .clone()
            .setup_graphics(sprite_render, &mut commands, &state_wrapper.0);
    }*/
}
