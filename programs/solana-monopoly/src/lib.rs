use bolt_lang::prelude::*;

declare_id!("GPE95gXdq1Rv6obrv3kfJA4C3W6z27oRZhV8PEHwL7BK");

#[program]
pub mod solana_monopoly {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
