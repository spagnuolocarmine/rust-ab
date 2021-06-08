use crate::engine::schedule::Schedule;
use std::any::Any;

pub trait State: Send + Sync + 'static {
    //fn new() -> Self;
    fn reset(&mut self);
    fn update(&mut self, _step: usize) {}
    fn as_any(&self) -> &dyn Any;
    fn before_step(&mut self, schedule: &mut Schedule);
    fn after_step(&mut self, schedule: &mut Schedule);
}
