pub trait State {
    fn new() -> Self;
    fn update(&mut self, _step: usize) {}
}
