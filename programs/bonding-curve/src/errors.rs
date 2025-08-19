use anchor_lang::prelude::*;

#[error_code]
pub enum CurveError {
    #[msg("Math overflow/underflow")]
    MathOverflow,
    #[msg("Division by zero")]
    DivByZero,
    #[msg("Insufficient output amount(slippage")]
    InsufficientOut,
    #[msg("Insufficient input amount")]
    InsufficientIn,
    #[msg("Insufficient inventory remaining on curve")]
    InsufficientInventory,
    #[msg("Curve already graduated")]
    Graduated,
    #[msg("Invalid account binding")]
    BadAccount,
    #[msg("Invalid fee bps")]
    BadFee,
}
