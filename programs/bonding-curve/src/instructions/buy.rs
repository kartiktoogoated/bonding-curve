use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use crate::{
    constants::*,
    errors::CurveError,
    events::{BuyExecuted, Graduated},
    math::{cpmm_quote_buy_dx, mul_div},
    state::{Config, Curve},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BuyArgs {
    pub max_pay_lamports: u64, // Maximum SOL user is willing to spend
    pub min_tokens_out: u64,   // Minimum tokens expected (slippage guard)
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(seeds = [SEED_CONFIG.as_bytes()], bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [SEED_CURVE.as_bytes(), token_mint.key().as_ref()],
        bump,
        has_one = token_mint,
        has_one = sol_vault,
    )]
    pub curve: Account<'info, Curve>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = token_mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [SEED_VAULT.as_bytes(), token_mint.key().as_ref()],
        bump
    )]
    pub sol_vault: SystemAccount<'info>,

    /// CHECK:
    /// This is safe because we enforce `fee_recipient.key() == config.fee_recipient`
    /// via a constraint, so the account is guaranteed to be the expected address.
    #[account(
        mut,
        constraint = fee_recipient.key() == config.fee_recipient @ CurveError::BadAccount
    )]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Buy>, args: BuyArgs) -> Result<()> {
    let buyer = &ctx.accounts.buyer;
    let config = &ctx.accounts.config;
    let curve = &mut ctx.accounts.curve;

    require!(!curve.graduated, CurveError::Graduated);

    let (dy_tokens, fee_lamports, dx_net_lamports, y_after_scaled) = cpmm_quote_buy_dx(
        curve.k_scaled,
        curve.y_v_scaled,
        args.max_pay_lamports,
        config.buy_fee_bps,
    )?;

    require!(
        dy_tokens >= args.min_tokens_out,
        CurveError::InsufficientOut
    );

    if fee_lamports > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: buyer.to_account_info(),
                    to: ctx.accounts.fee_recipient.to_account_info(),
                },
            ),
            fee_lamports,
        )?;
    }

    if dx_net_lamports > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: buyer.to_account_info(),
                    to: ctx.accounts.sol_vault.to_account_info(),
                },
            ),
            dx_net_lamports,
        )?;
    }

    let mint_key = ctx.accounts.token_mint.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        SEED_MAUTH.as_bytes(),
        mint_key.as_ref(),
        &[curve.bumps.mint_auth_bump],
    ]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: curve.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token::mint_to(cpi_ctx, dy_tokens)?;

    curve.y_v_scaled = y_after_scaled;
    curve.tokens_sold = curve
        .tokens_sold
        .checked_add(dy_tokens)
        .ok_or(CurveError::MathOverflow)?;

    emit!(BuyExecuted {
        mint: ctx.accounts.token_mint.key(),
        buyer: buyer.key(),
        dx_lamports: (dx_net_lamports as u64)
            .checked_add(fee_lamports)
            .ok_or(CurveError::MathOverflow)?,
        fee_lamports,
        dy_tokens,
        x_v_after: mul_div(curve.k_scaled, 1, y_after_scaled)?,
        y_v_after: y_after_scaled,
    });

    if curve.tokens_sold >= curve.curve_supply_cap {
        curve.graduated = true;
        emit!(Graduated {
            mint: ctx.accounts.token_mint.key(),
            tokens_sold: curve.tokens_sold,
            x_v_final: mul_div(curve.k_scaled, 1, curve.y_v_scaled)?,
            y_v_final: curve.y_v_scaled,
        });
    }

    Ok(())
}
