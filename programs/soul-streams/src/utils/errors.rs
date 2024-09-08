use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Amount cannot be zero")]
    ZeroAmount,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Duration cannot be zero")]
    ZeroDuration,
    #[msg("Amount to emit is zero")]
    ZeroAmountToEmit,
    #[msg("Stream hasn't ended yet")]
    OngoingStream,
}
