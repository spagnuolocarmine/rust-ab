use bevy::prelude::{Commands, SpriteBundle};

use crate::bevy::prelude::{Quat, Transform, Vec3};
use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
use crate::engine::state::State;
use crate::visualization::agent_render::{AgentRender, SpriteType};
use crate::visualization::asset_handle_factory::AssetHandleFactoryResource;
use crate::visualization::simulation_descriptor::SimulationDescriptor;

/// A simple trait which lets the developer set up the visualization components of his simulation.
/// This method will be called in a Bevy startup system.
pub trait VisualizationState<S: State>: Send + Sync {
    /// The method that will be called during the visualization inizialization.
    ///
    /// # Arguments
    ///
    /// * `commands` - Bevy [Commands](bevy::prelude::Commands), used mainly to create entities.
    /// * `sprite_render_factory` - A [bundle](crate::visualization::sprite_render_factory::SpriteFactoryResource) offering sprite-related resources.
    /// * `state` - The state of the simulation, available as a resource.
    /// * `schedule` - The schedule of the simulation, available as a resource.
    /// * `sim` - Data related to the simulation, for example width, height and center x/y.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_ab::visualization::visualization_state::VisualizationState;
    /// use bevy::prelude::{Commands, ResMut, Visible};
    /// use rust_ab::visualization::asset_handle_factory::AssetHandleFactoryResource;
    /// use rust_ab::visualization::simulation_descriptor::SimulationDescriptor;
    /// use rust_ab::visualization::agent_render::{SpriteType, AgentRender};
    /// # use rust_ab::engine::state::State;
    /// # use rust_ab::engine::agent::Agent;
    /// use rust_ab::bevy::prelude::Transform;
    /// use rust_ab::engine::schedule::Schedule;
    ///
    /// # struct MyState;
    /// # impl State for MyState{};
    ///
    /// # #[derive(Clone, Copy)]
    /// # struct MyAgent;
    /// # impl Agent for MyAgent{
    /// #    type SimState = MyState;
    /// #    fn step(&mut self,state: &Self::SimState) {}
    /// # }
    ///
    /// # impl AgentRender for MyAgent{
    /// #   fn sprite(&self) -> SpriteType {
    /// #       SpriteType::Emoji(String::from("bird"))
    /// #   }
    /// #   fn position(&self,state: &Self::SimState) -> (f32, f32, f32) {
    /// #       (0.,0.,0.)
    /// #   }
    /// #   fn scale(&self) -> (f32, f32) {
    /// #       (1.,1.)
    /// #   }
    /// #   fn rotation(&self) -> f32 {
    /// #       0.
    /// #   }
    /// #   fn update(&mut self,transform: &mut Transform,state: &Self::SimState, visible: &mut Visible) {
    /// #       
    /// #   }
    /// # }
    /// pub struct VisState;
    ///
    /// impl VisualizationState<MyAgent> for VisState {
    ///     fn on_init(&self, mut commands: Commands, mut sprite_render_factory: AssetHandleFactoryResource, mut state: ResMut<MyState>, mut schedule: ResMut<Schedule<MyAgent>>, mut sim: ResMut<SimulationDescriptor>) {
    ///         let agent = MyAgent;
    ///         schedule.schedule_repeating(agent, 0., 0);
    ///
    ///         let SpriteType::Emoji(emoji_code) = agent.sprite();
    ///         let sprite_render =
    ///             sprite_render_factory.get_emoji_loader(emoji_code);
    ///         agent.setup_graphics(sprite_render, &mut commands, &state);
    ///     }
    /// }
    /// ```
    fn on_init(
        &self,
        commands: &mut Commands,
        sprite_render_factory: &mut AssetHandleFactoryResource,
        state: &mut S,
        schedule: &mut Schedule,
        sim: &mut SimulationDescriptor,
    );

    fn setup_graphics(
        &self,
        schedule: &mut Schedule,
        commands: &mut Commands,
        state: &mut S,
        mut sprite_render_factory: AssetHandleFactoryResource,
    ) {
        for (agent_impl, _) in schedule.events.lock().unwrap().iter() {
            let agent_render = self.get_agent_render(&agent_impl.agent, state);
            let boxed_state = Box::new(state.as_state());
            let SpriteType::Emoji(emoji_code) =
                agent_render.sprite(&agent_impl.agent, &boxed_state);
            let sprite_render = sprite_render_factory.get_emoji_loader(emoji_code);
            self.setup_agent_graphics(
                &agent_impl.agent,
                agent_render,
                sprite_render,
                commands,
                &boxed_state,
            );
            //agent_render.setup_graphics(agent_impl.agent, sprite_render, &mut commands, &state);
        }
    }

    fn setup_agent_graphics(
        &self,
        agent: &Box<dyn Agent>,
        agent_render: Box<dyn AgentRender>,
        mut sprite_bundle: SpriteBundle,
        commands: &mut Commands,
        state: &Box<&dyn State>,
    ) {
        // AgentVis separate object which accepts an agent reference
        let (x, y, z) = agent_render.position(agent, state);
        let (scale_x, scale_y) = agent_render.scale(agent, state);
        let rotation = agent_render.rotation(agent, state);

        let mut transform = Transform::from_translation(Vec3::new(x, y, z));
        transform.scale.x = scale_x;
        transform.scale.y = scale_y;
        transform.rotation = Quat::from_rotation_z(rotation);

        sprite_bundle.transform = transform;
        commands
            .spawn()
            .insert(agent_render)
            .insert(transform)
            .insert_bundle(sprite_bundle);
    }

    /// The user must specify which AgentRender is associated to which Agent through this method
    /// TODO: how can the developer connect the two? Type string identifier?
    fn get_agent_render(&self, agent: &Box<dyn Agent>, state: &mut S) -> Box<dyn AgentRender>;

    fn get_agent(
        &self,
        agent_render: &Box<dyn AgentRender>,
        state: &Box<&dyn State>,
    ) -> Option<Box<dyn Agent>>;

    fn before_render(
        &mut self,
        state: &S,
        schedule: &Schedule,
        commands: &mut Commands,
        sprite_factory: &mut AssetHandleFactoryResource,
    ) {
    }
}
