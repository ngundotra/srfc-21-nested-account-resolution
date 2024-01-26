use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::Token,
};
use solana_program::{instruction::Instruction, program::invoke};

use crate::{
    LIBREPLEX_FAIR_LAUNCH, LIBREPLEX_INSCRIPTIONS, MPL_TOKEN_METADATA, SYSVAR_INSTRUCTIONS,
};

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct DeployV2Input {
    pub require_creator_cosign: bool,
    pub use_inscriptions: bool,
}
pub static METADATA_PREFIX: &str = "metadata";
pub static MASTER_EDITION_PREFIX: &str = "edition";
/*
    Deploy takes no input parameters as all of the
    string parameter + decimals have already been set by
    initialise.

    Deploy creates all on-chain objects (inscriptions,
    mints + any metadata) that are required to keep track of the
    launch lifecycle.
*/
#[derive(Accounts)]
pub struct DeployLegacyV2Ctx<'info> {
    #[account(mut)]
    // pub deployment: Account<'info, Deployment>,
    /// CHECK:
    pub deployment: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub fungible_mint: Signer<'info>,

    #[account(mut)]
    pub non_fungible_mint: Signer<'info>,

    #[account(seeds = ["hashlist".as_bytes(), 
    deployment.key().as_ref()],
    bump)]
    /// CHECK:
    pub hashlist: AccountInfo<'info>,

    /// CHECK: checked in code
    #[account(mut)]
    pub fungible_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub fungible_metadata: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_metadata: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_master_edition: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_token_account: UncheckedAccount<'info>,

    /* INTERACT WITH INSCRIPTIONS PROGRAM  */
    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub inscription_summary: UncheckedAccount<'info>,

    /// CHECK: passed in via CPI to libreplex_inscriptions program
    #[account(mut)]
    pub inscription_v3: UncheckedAccount<'info>,

    /// CHECK: passed in via CPI to libreplex_inscriptions program
    #[account(mut)]
    pub inscription_data: UncheckedAccount<'info>,

    /* BOILERPLATE PROGRAM ACCOUNTS */
    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: ID checked via constraint
    #[account(
        constraint = inscriptions_program.key() == LIBREPLEX_INSCRIPTIONS
    )]
    pub inscriptions_program: UncheckedAccount<'info>,

    #[account()]
    pub system_program: Program<'info, System>,

    /// CHECK: Id checked in constraint
    #[account(
        constraint = metadata_program.key() == MPL_TOKEN_METADATA
    )]
    #[account()]
    pub metadata_program: UncheckedAccount<'info>,

    /// CHECK: Id checked in constraint
    #[account(
        constraint = sysvar_instructions.key() == SYSVAR_INSTRUCTIONS
    )]
    #[account()]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct DeployLegacyV2CtxReadonly<'info> {
    #[account(mut)]
    // pub deployment: Account<'info, Deployment>,
    /// CHECK:
    pub deployment: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /* INITIALISE FUNGIBLE ACCOUNTS */
    #[account(mut)]
    pub fungible_mint: Signer<'info>,

    /* INITIALISE NON_FUNGIBLE ACCOUNTS. NB: no token account neede until mint */
    #[account(mut)]
    pub non_fungible_mint: Signer<'info>,
}

pub fn deploy(ctx: Context<DeployLegacyV2Ctx>) -> Result<()> {
    let data = &solana_program::hash::hashv(&[b"global:deploy_v2"]).to_bytes()[0..8];
    let ix = Instruction {
        program_id: LIBREPLEX_FAIR_LAUNCH,
        accounts: vec![
            AccountMeta::new(*ctx.accounts.deployment.key, true),
            AccountMeta::new(*ctx.accounts.hashlist.key, false),
            AccountMeta::new(*ctx.accounts.payer.key, true),
            AccountMeta::new(*ctx.accounts.payer.key, true),
            AccountMeta::new(*ctx.accounts.fungible_mint.key, true),
            AccountMeta::new(*ctx.accounts.fungible_escrow_token_account.key, true),
            AccountMeta::new(*ctx.accounts.fungible_metadata.key, true),
            AccountMeta::new(*ctx.accounts.non_fungible_mint.key, true),
            AccountMeta::new(*ctx.accounts.non_fungible_metadata.key, true),
            AccountMeta::new(*ctx.accounts.non_fungible_master_edition.key, true),
            AccountMeta::new(*ctx.accounts.non_fungible_token_account.key, true),
            AccountMeta::new(*ctx.accounts.inscription_summary.key, true),
            AccountMeta::new(*ctx.accounts.inscription_v3.key, true),
            AccountMeta::new(*ctx.accounts.inscription_data.key, true),
            AccountMeta::new(*ctx.accounts.token_program.key, false),
            AccountMeta::new(*ctx.accounts.associated_token_program.key, false),
            AccountMeta::new(*ctx.accounts.inscriptions_program.key, false),
            AccountMeta::new(*ctx.accounts.system_program.key, false),
            AccountMeta::new(*ctx.accounts.metadata_program.key, false),
            AccountMeta::new(*ctx.accounts.sysvar_instructions.key, false),
        ],
        data: data.to_vec(),
    };
    invoke(
        &ix,
        &[
            ctx.accounts.deployment.clone(),
            ctx.accounts.hashlist.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.fungible_mint.to_account_info(),
            ctx.accounts.fungible_escrow_token_account.to_account_info(),
            ctx.accounts.fungible_metadata.to_account_info(),
            ctx.accounts.non_fungible_mint.to_account_info(),
            ctx.accounts.non_fungible_metadata.to_account_info(),
            ctx.accounts.non_fungible_master_edition.to_account_info(),
            ctx.accounts.non_fungible_token_account.to_account_info(),
            ctx.accounts.inscription_summary.to_account_info(),
            ctx.accounts.inscription_v3.to_account_info(),
            ctx.accounts.inscription_data.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
            ctx.accounts.inscriptions_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.metadata_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
        ],
    )?;
    Ok(())
}

fn get_metadata_address(mint_key: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            MPL_TOKEN_METADATA.as_ref(),
            mint_key.as_ref(),
        ],
        &MPL_TOKEN_METADATA,
    )
    .0
}

fn get_edition_address(mint_key: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            MPL_TOKEN_METADATA.as_ref(),
            mint_key.as_ref(),
            MASTER_EDITION_PREFIX.as_bytes(),
        ],
        &MPL_TOKEN_METADATA,
    )
    .0
}

pub fn preflight_deploy(ctx: Context<DeployLegacyV2CtxReadonly>) -> Result<()> {
    let mut requested_accounts = AdditionalAccounts::new();
    let deployment = &ctx.accounts.deployment;
    let payer = &ctx.accounts.payer;
    let fungible_mint = &ctx.accounts.fungible_mint;
    let non_fungible_mint = &ctx.accounts.non_fungible_mint;

    let hashlist = Pubkey::find_program_address(
        &["hashlist".as_bytes(), deployment.key.as_ref()],
        &LIBREPLEX_FAIR_LAUNCH,
    )
    .0;

    let fungible_escrow_token_account = get_associated_token_address(&hashlist, fungible_mint.key);
    let non_fungible_token_account = get_associated_token_address(payer.key, non_fungible_mint.key);

    requested_accounts.add_account(&hashlist, false)?;
    requested_accounts.add_account(&fungible_escrow_token_account, true)?;
    requested_accounts.add_account(&get_metadata_address(fungible_mint.key), true)?;
    requested_accounts.add_account(&get_metadata_address(non_fungible_mint.key), true)?;
    requested_accounts.add_account(&get_edition_address(non_fungible_mint.key), true)?;
    requested_accounts.add_account(&non_fungible_token_account, true)?;
    // inscription summary
    requested_accounts.add_account(
        &Pubkey::find_program_address(&[b"inscription_summary"], &LIBREPLEX_INSCRIPTIONS).0,
        true,
    )?;
    // inscription v3
    requested_accounts.add_account(
        &Pubkey::find_program_address(
            &[b"inscription_v3", non_fungible_mint.key().as_ref()],
            &LIBREPLEX_INSCRIPTIONS,
        )
        .0,
        true,
    )?;
    // inscription data
    requested_accounts.add_account(
        &Pubkey::find_program_address(
            &[
                "inscription_data".as_bytes(),
                non_fungible_mint.key().as_ref(),
            ],
            &LIBREPLEX_INSCRIPTIONS,
        )
        .0,
        true,
    )?;
    requested_accounts.add_account(&Token::id(), false)?;
    requested_accounts.add_account(&AssociatedToken::id(), false)?;
    requested_accounts.add_account(&LIBREPLEX_INSCRIPTIONS, false)?;
    requested_accounts.add_account(&System::id(), false)?;
    requested_accounts.add_account(&MPL_TOKEN_METADATA, false)?;
    requested_accounts.add_account(&SYSVAR_INSTRUCTIONS, false)?;
    Ok(())
}
