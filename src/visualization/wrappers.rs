use std::marker::PhantomData;

use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
use crate::visualization::agent_render::AgentRender;
use crate::visualization::visualization_state::VisualizationState;

/// A wrapper of the currently active state, used as a Bevy resource.
pub struct ActiveState<A: 'static + Agent + AgentRender + Clone>(pub A::SimState);
/// A wrapper of the currently active schedule, used as a Bevy resource.
pub struct ActiveSchedule<A: 'static + Agent + AgentRender + Clone>(pub Schedule<A>);
/// Initialization method to set up state and agents, wrapped as a Bevy resource.
pub struct Initializer<A: AgentRender + Clone, I: VisualizationState<A> + 'static>(
    pub I,
    pub PhantomData<A>,
);
