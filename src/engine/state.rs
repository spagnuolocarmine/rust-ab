use crate::engine::agent::Agent;
use crate::engine::schedule::Schedule;
use std::any::Any;
pub trait State {
    fn update(&mut self, step: usize) {}
    fn init(&mut self, schedule: &mut Schedule);
    fn as_any(&mut self) -> &mut dyn Any;
    fn as_state(&mut self) -> &mut dyn State; 
}
