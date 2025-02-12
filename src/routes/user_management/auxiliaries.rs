use std::fmt::Debug;

use crate::auxiliaries::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum UpdateProfileError {
    #[error(transparent)]
    InvalidValue(#[from] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] sqlx::Error),
}

impl Debug for UpdateProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
