use anchor_lang::prelude::*;

use crate::state::MetadataInfo;

#[derive(Accounts)]
pub struct Token2022UpdateField<'info> {
    #[account(mut, has_one=update_authority)]
    pub metadata: Account<'info, MetadataInfo>,
    pub update_authority: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Field {
    /// The name field, corresponding to `TokenMetadata.name`
    Name,
    /// The symbol field, corresponding to `TokenMetadata.symbol`
    Symbol,
    /// The uri field, corresponding to `TokenMetadata.uri`
    Uri,
    /// A user field, whose key is given by the associated string
    Key(String),
}

pub fn t22_update_field<'info>(
    ctx: Context<'_, '_, '_, 'info, Token2022UpdateField<'info>>,
    field: Field,
    value: String,
) -> Result<()> {
    msg!("Not implemented. Cannot update fields");
    Ok(())
}
