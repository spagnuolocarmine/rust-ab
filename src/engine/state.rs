use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
pub trait State{
    type AgentToSchedule :'static + Agent + Clone + Send;
    fn update(&mut self, step: usize) {}
    fn init(&mut self, schedule: &mut Schedule<Self::AgentToSchedule>){}
}
