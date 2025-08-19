use crate::constants::*;
use anchor_lang::prelude::*;

// Global Config: admin, fees and policy flags
#[account]
pub struct Config {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub buy_fee_bps: u16,
    pub sell_fee_bps: u16,
    pub allow_sell_pre_grad: bool,
    pub _padding: [u8; 7],
}

impl Config {
    pub const LEN: usize = 8 + 32 + 32 + 2 + 2 + 1 + 7;
    pub const SEED: &'static str = SEED_CONFIG;
}
