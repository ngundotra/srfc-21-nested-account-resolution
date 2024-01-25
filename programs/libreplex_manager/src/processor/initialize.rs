use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;
use bytemuck::bytes_of;
use solana_program::{
    instruction::Instruction,
    program::{invoke, set_return_data},
};

use crate::LIBREPLEX_FAIR_LAUNCH;

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK:
    #[account(address = LIBREPLEX_FAIR_LAUNCH)]
    pub libreplex_fair_launch: AccountInfo<'info>,

    /// CHECK: checked by libreplex fair launch
    pub deployment: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitialiseInputV2 {
    pub max_number_of_tokens: u64, // this is the max *number* of tokens
    pub decimals: u8,
    pub ticker: String,
    // pub deployment_template: String,
    // pub mint_template: String,
    pub offchain_url: String, // used both for the fungible and the non-fungible
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct _InitialiseInputV2 {
    pub limit_per_mint: u64, // this number of SPL tokens are issued into the escrow when an op: 'mint' comes in
    pub max_number_of_tokens: u64, // this is the max *number* of tokens
    pub decimals: u8,
    pub ticker: String,
    pub deployment_template: String,
    pub mint_template: String,
    pub offchain_url: String, // used both for the fungible and the non-fungible
    pub require_creator_cosign: bool,
    pub use_inscriptions: bool,
    pub deployment_type: u8,
}

// pub deployment_type: u8 = 0; for NFT
// pub deployment_type: u8 = 2; for cNFT

#[derive(Accounts)]
pub struct InitializeReadonly<'info> {
    pub payer: Signer<'info>,
}

pub fn initialize(ctx: Context<Initialize>, args: InitialiseInputV2) -> Result<()> {
    let mint_template = format!(
        "{{\"p\":\"spl-20\",\"op\":\"mint\",\"tick\":\"{}\",\"amt\":\"1\"}}",
        args.ticker
    );

    let real_args = _InitialiseInputV2 {
        limit_per_mint: 1,
        max_number_of_tokens: args.max_number_of_tokens,
        decimals: args.decimals,
        ticker: args.ticker,
        deployment_template: "".to_string(),
        mint_template,
        offchain_url: args.offchain_url,
        require_creator_cosign: false,
        use_inscriptions: true,
        deployment_type: 0,
    };

    let disc = &solana_program::hash::hashv(&[b"global:initialise_v2"]).to_bytes()[0..8];
    let serialized_data = real_args.try_to_vec()?;
    let data = [&disc[..], &serialized_data].concat();
    let ix = Instruction {
        program_id: LIBREPLEX_FAIR_LAUNCH,
        accounts: vec![
            AccountMeta::new(*ctx.accounts.deployment.key, true),
            AccountMeta::new(*ctx.accounts.payer.key, true),
            AccountMeta::new(*ctx.accounts.payer.key, true),
            AccountMeta::new(*ctx.accounts.system_program.key, false),
        ],
        data,
    };
    invoke(
        &ix,
        &[
            ctx.accounts.deployment.clone(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    Ok(())
}

pub fn preflight_initialize(
    ctx: Context<InitializeReadonly>,
    args: InitialiseInputV2,
) -> Result<()> {
    let mut requested_accounts = AdditionalAccounts::new();

    let deployment = Pubkey::find_program_address(
        &["deployment".as_ref(), args.ticker.as_ref()],
        &LIBREPLEX_FAIR_LAUNCH,
    )
    .0;

    requested_accounts.add_account(&deployment, true)?;
    requested_accounts.add_account(ctx.accounts.payer.key, true)?;
    requested_accounts.add_account(ctx.accounts.payer.key, true)?;
    requested_accounts.add_account(&System::id(), false)?;

    set_return_data(bytes_of(&requested_accounts));
    Ok(())
}
