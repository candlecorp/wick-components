#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    PayloadError(#[from] wasmrs_guest::PayloadError),
}
