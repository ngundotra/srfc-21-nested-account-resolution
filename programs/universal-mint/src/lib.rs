use anchor_lang::prelude::*;

pub mod processor;
pub mod state;

use processor::*;
use state::MetadataInfo;

use ::spl_token_metadata_interface::{borsh::BorshSerialize, state::TokenMetadata};
use anchor_lang::solana_program::program::set_return_data;
use anchor_spl::token_interface;

declare_id!("HfmoA2Urje3qNQ2f9jRuMHepz1aqhG4h6HLeiyntRCe6");

#[derive(Accounts)]
pub struct Token2022Emitter<'info> {
    pub metadata: Account<'info, MetadataInfo>,
    // pub metadata_pointer: AccountInfo<'info>,
}

/// Universal program to mint, transfer, and close mints of
/// SPL token, SPL token 2022, SPL token 2022 metadata
#[program]
pub mod universal_mint {
    use super::*;

    /// Testing only
    pub fn create_spl_token(ctx: Context<CreateSplToken>, decimals: u8) -> Result<()> {
        processor::create_spl_token(ctx, decimals)
    }

    /// Testing only
    pub fn create_spl_token_extension(ctx: Context<CreateSplToken22>, decimals: u8) -> Result<()> {
        processor::create_spl_token_extension(ctx, decimals)
    }

    /// Create an SPL Token Extension mint with metadata
    /// and mint your self the only in circulation
    pub fn create_spl_token_extension_metadata(
        ctx: Context<CreateSplToken22Metadata>,
        name: String,
        description: String,
    ) -> Result<()> {
        processor::create_spl_token_extension_metadata(ctx, name, description)
    }

    pub fn preflight_create_spl_token_extension_metadata(
        ctx: Context<CreateSplToken22MetadataReadonly>,
        name: String,
        description: String,
    ) -> Result<()> {
        processor::preflight_create_spl_token_extension_metadata(ctx, name, description)
    }

    /// Transfers ownership of 1 amount from the owner to the destination
    pub fn transfer_token<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferToken<'info>>,
        amount: u64,
    ) -> Result<()> {
        processor::transfer_token(ctx, amount)
    }

    pub fn preflight_transfer_token<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferTokenReadonly<'info>>,
        amount: u64,
    ) -> Result<()> {
        processor::preflight_transfer_token(ctx, amount)
    }

    // Describe endpoint

    /// Use this to get a human-readable interpretation of an account
    pub fn describe(ctx: Context<Describe>) -> Result<()> {
        processor::describe(ctx)
    }

    #[ix(namespace = "spl_token_metadata_interface", name = "emitter")]
    pub fn emitter<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022Emitter<'info>>,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<()> {
        let metadata = &ctx.accounts.metadata;
        let token_metadata = TokenMetadata {
            update_authority: Some(metadata.key()).try_into()?,
            mint: metadata.key(),
            uri: "".to_string(),
            name: "".to_string(),
            ..Default::default()
        };

        let metadata_bytes = token_metadata.try_to_vec()?;

        if let Some(range) = TokenMetadata::get_slice(&metadata_bytes, start, end) {
            set_return_data(range);
        }
        Ok(())
    }
}
