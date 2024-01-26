use anchor_lang::{prelude::*, solana_program::program::set_return_data};
use spl_token_metadata_interface::{borsh::BorshSerialize, state::TokenMetadata};

use crate::state::MetadataInfo;

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
        update_authority: Some(metadata.update_authority).try_into()?,
        mint: metadata.mint,
        uri: "a".to_string(),
        name: "b".to_string(),
        symbol: "c".to_string(),
        additional_metadata: vec![],
    };

    let metadata_bytes = token_metadata.try_to_vec()?;

    if let Some(range) = TokenMetadata::get_slice(&metadata_bytes, start, end) {
        set_return_data(range);
    }
    Ok(())
}
