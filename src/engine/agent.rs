use crate::engine::priority::Priority;
use crate::engine::schedule::ScheduleOptions;
use crate::engine::state::State;
use std::collections::HashMap;

pub trait Agent: Clone + Send + Sync {

    fn step(&mut self, state: &Box<dyn State>);

    /// Specifies whether this agent should be removed from the schedule after the current step.
    fn should_remove(&mut self, _state: &Box<dyn State>) -> bool {
        false
    }

    /// Allows the agent to schedule new agents without having direct access to the Schedule.
    /// This should NOT return an agent that is already scheduled.
    fn should_reproduce(
        &mut self,
        _state: &Box<dyn State>
    ) -> Option<HashMap<Box<Self>, ScheduleOptions>> {
        None
    }
}
