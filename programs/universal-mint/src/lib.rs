use anchor_lang::prelude::*;

pub mod processor;
pub mod state;

use processor::*;

use ::spl_token_metadata_interface::{borsh::BorshSerialize, state::TokenMetadata};
use anchor_spl::token_2022::ID as TOKEN_2022_PROGRAM_ID;
use anchor_spl::token_interface::{Mint, TokenAccount};

declare_id!("HfmoA2Urje3qNQ2f9jRuMHepz1aqhG4h6HLeiyntRCe6");

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
        symbol: String,
        uri: String,
        description: String,
    ) -> Result<()> {
        processor::create_spl_token_extension_metadata(ctx, name, symbol, uri, description)
    }

    pub fn preflight_create_spl_token_extension_metadata(
        ctx: Context<CreateSplToken22MetadataReadonly>,
        name: String,
        symbol: String,
        uri: String,
        description: String,
    ) -> Result<()> {
        processor::preflight_create_spl_token_extension_metadata(
            ctx,
            name,
            symbol,
            uri,
            description,
        )
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

    #[ix(
        namespace = "spl_token_metadata_interface",
        name = "update_the_authority"
    )]
    pub fn t22_update_authority<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022UpdateAuthority<'info>>,
        new_authority: Option<Pubkey>,
    ) -> Result<()> {
        processor::t22_update_authority(ctx, new_authority)
    }

    #[ix(namespace = "spl_token_metadata_interface", name = "emitter")]
    pub fn t22_emitter<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022Emitter<'info>>,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<()> {
        processor::t22_emitter(ctx, start, end)
    }

    #[ix(namespace = "spl_token_metadata_interface", name = "remove_key_ix")]
    pub fn t22_remove_key<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022RemoveKey<'info>>,
        idempotent: bool,
        key: String,
    ) -> Result<()> {
        processor::t22_remove_key(ctx, idempotent, key)
    }

    #[ix(namespace = "spl_token_metadata_interface", name = "updating_field")]
    pub fn t22_update_field<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022UpdateField<'info>>,
        field: Field,
        value: String,
    ) -> Result<()> {
        processor::t22_update_field(ctx, field, value)
    }

    #[ix(
        namespace = "spl_token_metadata_interface",
        name = "initialize_account"
    )]
    pub fn t22_initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Token2022Initialize<'info>>,
    ) -> Result<()> {
        processor::t22_initialize(ctx)
    }
}
