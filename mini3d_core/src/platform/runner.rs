use crate::app::App;

pub trait Runner {
    fn invoke(&mut self, app: App);
}