use anchor_lang::prelude::*;

declare_id!("4SKsW4jTMPcdetYPjFqvZw76LYYcgAJXZEdtxHHXe9vN");

#[program]
pub mod soul_streams {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
