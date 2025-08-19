use anchor_lang::prelude::*;

// Emitted on successful buy (SOL -> Token)
#[event]
pub struct BuyExecuted {
    pub mint: Pubkey,
    pub buyer: Pubkey,
    pub dx_lamports: u64, // gross paid by user (before fee)
    pub fee_lamports: u64,
    pub dy_tokens: u64,  // tokens received by user
    pub x_v_after: u128, // scaled reserves after trade
    pub y_v_after: u128,
}

// Emitted on successful sell (token -> SOL)
#[event]
pub struct SellExecuted {
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub dy_tokens: u64,
    pub fee_lamports: u64,
    pub dx_lamports: u64,
    pub x_v_after: u128,
    pub y_v_after: u128,
}

// Emitted when curve inventory is fully sold and trading is closed
#[event]
pub struct Graduated {
    pub mint: Pubkey,
    pub tokens_sold: u64,
    pub x_v_final: u128,
    pub y_v_final: u128,
}
