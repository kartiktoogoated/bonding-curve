// Fixed point scale used for all curve math

pub const SCALE: u128 = 10_000;

// Basis point denominator 10_000 bps = 100.00%
pub const BPS_DENOMINATOR: u16 = 10_000;

// PDA seed strings
pub const SEED_CONFIG: &str = "config";
pub const SEED_CURVE: &str = "curve";
pub const SEED_VAULT: &str = "vault";
pub const SEED_MAUTH: &str = "mint_auth";
