use crate::constants::*;
use crate::errors::CurveError;
use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Args {
    pub fee_recipient: Pubkey,
    pub buy_fee_bps: u16,
    pub sell_fee_bps: u16,
    pub allow_sell_pre_grad: bool,
}

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [SEED_CONFIG.as_bytes()],
        bump,
        space = Config::LEN
        )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitConfig>, args: Args) -> Result<()> {
    require!(args.buy_fee_bps <= BPS_DENOMINATOR, CurveError::BadFee);
    require!(args.sell_fee_bps <= BPS_DENOMINATOR, CurveError::BadFee);

    let cfg = &mut ctx.accounts.config;
    cfg.admin = ctx.accounts.admin.key();
    cfg.fee_recipient = args.fee_recipient;
    cfg.buy_fee_bps = args.buy_fee_bps;
    cfg.sell_fee_bps = args.sell_fee_bps;
    cfg.allow_sell_pre_grad = args.allow_sell_pre_grad;
    cfg._padding = [0u8; 7];

    Ok(())
}
