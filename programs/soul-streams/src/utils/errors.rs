use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Amount cannot be zero")]
    ZeroAmount,
    #[msg("Duration cannot be zero")]
    ZeroDuration,
}
