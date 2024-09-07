use anchor_lang::prelude::*;

#[event]
pub struct Initialized {
    pub stream_count: u64,
}

#[event]
pub struct NewStreamCreated {
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub starting_timestamp: u64,
    pub duration: u64,
}
