use crate::constants::*;
use crate::errors::CurveError;
use crate::state::{Config, Curve, CurveBumps};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, spl_token::instruction::AuthorityType, Mint, Token};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitCurveArgs {
    pub x_v0_lamports: u64,
    pub y_v0_tokens: u64,
    pub curve_supply_cap: u64,
    pub take_mint_authority: bool,
}

#[derive(Accounts)]
pub struct InitCurve<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_CONFIG.as_bytes()],
        bump
        )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [SEED_CURVE.as_bytes(), token_mint.key().as_ref()],
        bump,
        space = Curve::LEN
        )]
    pub curve: Account<'info, Curve>,

    /// CHECK: This is a PDA that will be initialized
    #[account(
        init,
        payer = admin,
        seeds = [SEED_VAULT.as_bytes(), token_mint.key().as_ref()],
        bump,
        space = 0
        )]
    pub sol_vault: UncheckedAccount<'info>,

    pub current_mint_authority: Option<Signer<'info>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitCurve>, args: InitCurveArgs) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.config.admin,
        ctx.accounts.admin.key(),
        CurveError::BadAccount
    );
    require!(args.curve_supply_cap > 0, CurveError::InsufficientInventory);
    require!(
        args.x_v0_lamports > 0 && args.y_v0_tokens > 0,
        CurveError::InsufficientIn
    );

    let (_curve_pda, curve_bump) = Pubkey::find_program_address(
        &[
            SEED_CURVE.as_bytes(),
            ctx.accounts.token_mint.key().as_ref(),
        ],
        ctx.program_id,
    );
    let (_vault_pda, vault_bump) = Pubkey::find_program_address(
        &[
            SEED_VAULT.as_bytes(),
            ctx.accounts.token_mint.key().as_ref(),
        ],
        ctx.program_id,
    );
    let (mint_auth_pda, mint_auth_bump) = Pubkey::find_program_address(
        &[
            SEED_MAUTH.as_bytes(),
            ctx.accounts.token_mint.key().as_ref(),
        ],
        ctx.program_id,
    );
    let x_v_scaled: u128 = (args.x_v0_lamports as u128)
        .checked_mul(SCALE)
        .ok_or(CurveError::MathOverflow)?;
    let y_v_scaled: u128 = (args.y_v0_tokens as u128)
        .checked_mul(SCALE)
        .ok_or(CurveError::MathOverflow)?;
    let k_scaled: u128 = x_v_scaled
        .checked_mul(y_v_scaled)
        .ok_or(CurveError::MathOverflow)?;

    let c = &mut ctx.accounts.curve;
    c.token_mint = ctx.accounts.token_mint.key();
    c.sol_vault = ctx.accounts.sol_vault.key();
    c.mint_auth = mint_auth_pda;

    c.x_v_scaled = x_v_scaled;
    c.y_v_scaled = y_v_scaled;
    c.k_scaled = k_scaled;
    c.scale = SCALE;

    c.curve_supply_cap = args.curve_supply_cap;
    c.tokens_sold = 0;
    c.graduated = false;

    c.bumps = CurveBumps {
        curve_bump,
        vault_bump,
        mint_auth_bump,
    };
    if args.take_mint_authority {
        let current_auth = ctx
            .accounts
            .current_mint_authority
            .as_ref()
            .ok_or(CurveError::BadAccount)?;

        // CPI: token::set_authority(mint, current_auth_signer, NewAuthority = mint_auth_pda)
        let cpi_accounts = token::SetAuthority {
            account_or_mint: ctx.accounts.token_mint.to_account_info(),
            current_authority: current_auth.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::set_authority(cpi_ctx, AuthorityType::MintTokens, Some(mint_auth_pda))?;
    }

    Ok(())
}
