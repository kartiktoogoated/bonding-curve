use crate::constants::*;
use anchor_lang::prelude::*;

// Bumps for PDAs used by curve
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CurveBumps {
    pub curve_bump: u8,
    pub vault_bump: u8,
    pub mint_auth_bump: u8,
}

// Onchain state for a single bonding curve instance
#[account]
pub struct Curve {
    // Binding
    pub token_mint: Pubkey,
    pub sol_vault: Pubkey,
    pub mint_auth: Pubkey,

    // Virtual reserves & invariant (scaled by SCALE)
    pub x_v_scaled: u128,
    pub y_v_scaled: u128,
    pub k_scaled: u128,
    pub scale: u128,

    // Inventory management
    pub curve_supply_cap: u64,
    pub tokens_sold: u64,
    pub graduated: bool,

    pub bumps: CurveBumps,
}

impl Curve {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 16 + 16 + 16 + 16 + 8 + 8 + 1 + 3;
    pub const SEED: &'static str = SEED_CURVE;
    pub const VAULT_SEED: &'static str = SEED_VAULT;
    pub const MINT_AUTH_SEED: &'static str = SEED_MAUTH;
}

