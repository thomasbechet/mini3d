use mini3d_derive::Serialize;

#[derive(Serialize)]
pub enum SystemEvent {
    Shutdown,
}