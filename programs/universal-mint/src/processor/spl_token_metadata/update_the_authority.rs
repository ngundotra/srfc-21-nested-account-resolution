use anchor_lang::prelude::*;

use crate::state::MetadataInfo;

#[derive(Accounts)]
pub struct Token2022UpdateAuthority<'info> {
    #[account(mut, has_one=update_authority)]
    pub metadata: Account<'info, MetadataInfo>,
    pub update_authority: Signer<'info>,
}

pub fn t22_update_authority<'info>(
    ctx: Context<'_, '_, '_, 'info, Token2022UpdateAuthority<'info>>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let metadata = &mut ctx.accounts.metadata;
    metadata.update_authority = new_authority.unwrap_or(System::id());
    Ok(())
}
