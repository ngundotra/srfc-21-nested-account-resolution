use anchor_lang::prelude::*;

use crate::state::MetadataInfo;

#[derive(Accounts)]
pub struct Token2022RemoveKey<'info> {
    #[account(mut, has_one=update_authority)]
    pub metadata: Account<'info, MetadataInfo>,
    pub update_authority: Signer<'info>,
}

pub fn t22_remove_key<'info>(
    ctx: Context<'_, '_, '_, 'info, Token2022RemoveKey<'info>>,
    idempotent: bool,
    key: String,
) -> Result<()> {
    msg!("Not implemented. Cannot remove keys");
    Ok(())
}
