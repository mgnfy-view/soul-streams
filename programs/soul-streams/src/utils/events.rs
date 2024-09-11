use anchor_lang::prelude::*;

#[event]
pub struct Initialized {
    pub stream_count: u64,
}

#[event]
pub struct NewStreamCreated {
    pub stream: Pubkey,
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub starting_timestamp: u64,
    pub duration: u64,
    pub count: u64,
}

#[event]
pub struct AmountWithdrawnFromStream {
    pub stream: Pubkey,
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub amount_withdrawn: u64,
    pub count: u64,
}

#[event]
pub struct StreamCanceled {
    pub stream: Pubkey,
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub count: u64,
}

#[event]
pub struct StreamReplenished {
    pub stream: Pubkey,
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub starting_timestamp: u64,
    pub duration: u64,
}
