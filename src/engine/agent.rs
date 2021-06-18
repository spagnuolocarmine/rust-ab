use std::collections::HashMap;

use crate::engine::schedule::ScheduleOptions;
use crate::engine::state::State;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::DynClone;

pub trait Agent: Downcast + DynClone + Send + Sync {
    fn step(&mut self, state: &Box<&mut dyn State>);

    /// Specifies whether this agent should be removed from the schedule after the current step.
    fn should_remove(&mut self, _state: &Box<&mut dyn State>) -> bool {
        false
    }

    fn get_id(&self) -> u128;

    /*
    /// Allows the agent to schedule new agents without having direct access to the Schedule.
    /// This should NOT return an agent that is already scheduled.
    fn should_reproduce(
        &mut self,
        _state: &Box<&mut dyn State>,
    ) -> Option<HashMap<Box<dyn Agent>, ScheduleOptions>> {
        None
    }*/
}

dyn_clone::clone_trait_object!(Agent);
impl_downcast!(Agent);
