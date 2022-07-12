pub trait Runner: Sized {
    fn invoke(&mut self, app: App);
}