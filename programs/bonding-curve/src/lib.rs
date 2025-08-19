use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod events;
pub mod math;
pub mod state;

declare_id!("D1F2ffgFrSkDW8TdnWv8dsvCtKHRwQpimgpcYeFPVKF4");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
