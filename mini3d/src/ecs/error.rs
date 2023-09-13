use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("Container borrow mut")]
    ContainerBorrowMut,
}
