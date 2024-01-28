use anchor_lang::{prelude::*, solana_program::program::set_return_data};
use spl_token_metadata_interface::{borsh::BorshSerialize, state::TokenMetadata};

use crate::state::{get_program_authority, MetadataInfo};

#[derive(Accounts)]
pub struct Token2022Emitter<'info> {
    pub metadata: Account<'info, MetadataInfo>,
}

pub fn t22_emitter<'info>(
    ctx: Context<'_, '_, '_, 'info, Token2022Emitter<'info>>,
    start: Option<u64>,
    end: Option<u64>,
) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let token_metadata = TokenMetadata {
        update_authority: Some(get_program_authority().0).try_into()?,
        mint: metadata.mint,
        name: metadata.name.to_string(),
        symbol: metadata.symbol.to_string(),
        uri: metadata.uri.to_string(),
        additional_metadata: vec![("Description".to_string(), metadata.description.to_string())],
    };

    let metadata_bytes = token_metadata.try_to_vec()?;

    if let Some(range) = TokenMetadata::get_slice(&metadata_bytes, start, end) {
        set_return_data(range);
    }
    Ok(())
}
