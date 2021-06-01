use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
pub trait State{

    fn update(&mut self, step: usize) {}
    fn init(&mut self, schedule: &mut Schedule){}
}
