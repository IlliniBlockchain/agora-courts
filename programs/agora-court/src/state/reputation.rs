use anchor_lang::prelude::*;

#[account]
pub struct Reputation {
    // State parameters - will change
    reputation: u32,
    claim_queue: Vec<Pubkey>,
}

impl Reputation {
    // discriminator - 8 bytes
    pub const STATIC_SIZE: usize = 8 + 4;
}