use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;

pub use instructions::*;
pub use instructions::init_config::*;
pub use instructions::init_curve::*;

declare_id!("D1F2ffgFrSkDW8TdnWv8dsvCtKHRwQpimgpcYeFPVKF4");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn init_config(
        ctx: Context<InitConfig>,
        args: InitConfigArgs,
    ) -> Result<()> {
        instructions::init_config::handler(ctx, args)
    }

    pub fn init_curve(
        ctx: Context<InitCurve>,
        args: InitCurveArgs,
    ) -> Result<()> {
        instructions::init_curve::handler(ctx, args)
    }
}
