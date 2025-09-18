use crate::constants::*;
use crate::errors::CurveError;
use anchor_lang::prelude::*;

// Safe (a*b) / d with overflow checks
#[inline]
pub fn mul_div(a: u128, b: u128, d: u128) -> Result<u128> {
    a.checked_mul(b)
        .and_then(|p| p.checked_div(d))
        .ok_or(CurveError::MathOverflow.into())
}

// Recompute x_v from k and y_v: x = k/y
#[inline]
pub fn x_from_k_y(k_scaled: u128, y_scaled: u128) -> Result<u128> {
    require!(y_scaled > 0, CurveError::DivByZero);
    mul_div(k_scaled, 1, y_scaled)
}

/// Quote tokens_out for a given **gross** SOL in (lamports) and fee_bps.
/// Fees are **taken outside** the pool, so k remains constant.
/// Returns: (dy_tokens, fee_lamports, dx_net_scaled, y1_scaled)
pub fn cpmm_quote_buy_dx(
    k_scaled: u128,
    y0_scaled: u128,
    dx_gross_lamports: u64,
    fee_bps: u16,
) -> Result<(u64, u64, u64, u128)> {
    require!(fee_bps <= BPS_DENOMINATOR, CurveError::BadFee);
    require!(y0_scaled > 0, CurveError::DivByZero);

    // fee (lamports)
    let fee_lamports = (dx_gross_lamports as u128)
        .checked_mul(fee_bps as u128)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR as u128))
        .ok_or(CurveError::MathOverflow)? as u64;
    // net lamports sent into pool (scaled)
    let dx_net_lamports = dx_gross_lamports
        .checked_sub(fee_lamports)
        .ok_or(CurveError::InsufficientIn)?;
    let dx_net_scaled = (dx_net_lamports as u128)
        .checked_mul(SCALE)
        .ok_or(CurveError::MathOverflow)?;

    // A = k / y0   (scaled SOL)
    let a = mul_div(k_scaled, 1, y0_scaled)?;
    // y1 = k / (A + dx)
    let denom = a
        .checked_add(dx_net_scaled)
        .ok_or(CurveError::MathOverflow)?;
    let y1_scaled = mul_div(k_scaled, 1, denom)?;

    // dy = y0 - y1
    let dy_scaled = y0_scaled
        .checked_sub(y1_scaled)
        .ok_or(CurveError::MathOverflow)?;
    let dy_tokens = (dy_scaled / SCALE) as u64;

    Ok((dy_tokens, fee_lamports, dx_net_lamports, y1_scaled))
}

/// Quote SOL_out (net to user) for selling tokens_in.
/// Fees are **taken from payout** (outside the pool).
/// Returns: (dx_net_lamports, fee_lamports, y1_scaled)
pub fn cpmm_quote_sell_dy(
    k_scaled: u128,
    y0_scaled: u128,
    dy_tokens: u64,
    fee_bps: u16,
) -> Result<(u64, u64, u128)> {
    require!(fee_bps <= BPS_DENOMINATOR, CurveError::BadFee);
    require!(y0_scaled > 0, CurveError::DivByZero);

    let dy_scaled = (dy_tokens as u128)
        .checked_mul(SCALE)
        .ok_or(CurveError::MathOverflow)?;

    // x_before = k / y0
    let x_before = mul_div(k_scaled, 1, y0_scaled)?;
    // x_after  = k / (y0 + dy)
    let y1_scaled = y0_scaled
        .checked_add(dy_scaled)
        .ok_or(CurveError::MathOverflow)?;
    let x_after = mul_div(k_scaled, 1, y1_scaled)?;

    // dx_scaled = x_before - x_after   (scaled SOL)
    let dx_scaled = x_before
        .checked_sub(x_after)
        .ok_or(CurveError::MathOverflow)?;
    let dx_lamports = (dx_scaled / SCALE) as u64;

    // fee on payout
    let fee_lamports = (dx_lamports as u128)
        .checked_mul(fee_bps as u128)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR as u128))
        .ok_or(CurveError::MathOverflow)? as u64;

    let dx_net_lamports = dx_lamports
        .checked_sub(fee_lamports)
        .ok_or(CurveError::MathOverflow)?;
    Ok((dx_net_lamports, fee_lamports, y1_scaled))
}
