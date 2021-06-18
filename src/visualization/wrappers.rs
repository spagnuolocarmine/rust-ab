use std::marker::PhantomData;

use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
use crate::engine::state::State;
use crate::visualization::agent_render::AgentRender;
use crate::visualization::visualization_state::VisualizationState;

/// A wrapper of the currently active state, used as a Bevy resource.
pub struct ActiveState<S: State>(pub S);
/// A wrapper of the currently active schedule, used as a Bevy resource.
pub struct ActiveSchedule(pub Schedule);
/// Initialization method to set up state and agents, wrapped as a Bevy resource.
pub struct Initializer<I: VisualizationState<S> + 'static, S: State>(pub I, pub PhantomData<S>);
