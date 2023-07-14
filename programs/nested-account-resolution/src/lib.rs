use anchor_lang::prelude::*;

declare_id!("J5kQSQoRjuWYwPBKEWpMcttcZVU7f2WGLBQWFXNZCfVU");

#[program]
pub mod nested_account_resolution {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
