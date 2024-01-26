use crate::state::MetadataInfo;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;

use anchor_lang::solana_program::program::set_return_data;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token_interface::spl_token_2022::instruction::freeze_account;
use anchor_spl::token_interface::FreezeAccount;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::mint_to,
    token_2022::spl_token_2022::extension::metadata_pointer, token_interface::Token2022,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    token_2022::MintTo,
    token_interface::spl_token_2022::extension::ExtensionType,
};
use bytemuck::bytes_of;

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String, description: String)]
pub struct CreateSplToken22Metadata<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut)]
    mint: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    ata: AccountInfo<'info>,
    /// CHECK:
    #[account(seeds = ["AUTHORITY".as_bytes()], bump)]
    program_authority: AccountInfo<'info>,
    /// CHECK:
    #[account(init, space = 8 + 32 + 32 + 4 + name.len() + 4 + symbol.len() + 4 + uri.len() + 4 + description.len(), payer=payer, seeds=[&mint.key.to_bytes(), "token22".as_bytes(), &"metadata_pointer".as_bytes()], bump)]
    metadata_pointer: Account<'info, MetadataInfo>,
    token_program: Program<'info, Token2022>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateSplToken22MetadataReadonly<'info> {
    /// CHECK:
    payer: UncheckedAccount<'info>,
    /// CHECK:
    mint: UncheckedAccount<'info>,
}

pub fn preflight_create_spl_token_extension_metadata(
    ctx: Context<CreateSplToken22MetadataReadonly>,
    name: String,
    symbol: String,
    uri: String,
    description: String,
) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let mint = &ctx.accounts.mint;

    let mut accounts = AdditionalAccounts::new();

    let ata = get_associated_token_address_with_program_id(payer.key, mint.key, &Token2022::id());
    let program_authority =
        Pubkey::find_program_address(&[&"AUTHORITY".as_bytes()], &crate::id()).0;
    let metadata_pointer = Pubkey::find_program_address(
        &[
            &mint.key.to_bytes(),
            "token22".as_bytes(),
            &"metadata_pointer".as_bytes(),
        ],
        &crate::id(),
    )
    .0;

    let token_program = Token2022::id();
    let associated_token_program = AssociatedToken::id();
    let system_program = System::id();

    let to_check = &[
        (&ata, true),
        (&program_authority, false),
        (&metadata_pointer, true),
        (&token_program, false),
        (&associated_token_program, false),
        (&system_program, false),
    ];

    let mut last_idx = 0;
    for account in ctx.remaining_accounts.iter() {
        let (acc_to_check, mut_check) = to_check.get(last_idx).unwrap();

        if account.key != *acc_to_check {
            msg!("Missing {}", *acc_to_check);
            return Err(ProgramError::InvalidInstructionData.into());
        }

        if account.is_writable != *mut_check {
            msg!(
                "Expected {} to have isWritable: {}, but is {}",
                account.key,
                *mut_check,
                account.is_writable
            );
            return Err(ProgramError::InvalidInstructionData.into());
        }
        last_idx += 1;
    }

    for (acc, writability) in to_check[last_idx..].iter() {
        accounts.add_account(acc, *writability)?;
    }

    set_return_data(bytes_of(&accounts));
    Ok(())
}

pub fn create_spl_token_extension_metadata(
    ctx: Context<CreateSplToken22Metadata>,
    name: String,
    symbol: String,
    uri: String,
    description: String,
) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let mint = &ctx.accounts.mint;
    let ata = &ctx.accounts.ata;
    let program_authority = &ctx.accounts.program_authority;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;

    // Write to the metadata pointer
    let metadata_pointer = &mut ctx.accounts.metadata_pointer;
    metadata_pointer.update_authority = payer.key();
    metadata_pointer.mint = mint.key();
    metadata_pointer.name = name.clone();
    metadata_pointer.symbol = symbol.clone();
    metadata_pointer.uri = uri.clone();
    metadata_pointer.description = description.clone();

    let extension_len = ExtensionType::try_calculate_account_len::<
        anchor_spl::token_2022::spl_token_2022::state::Mint,
    >(&[ExtensionType::MetadataPointer])?;

    // msg!("Found size: {}", size);
    // let extension_len: usize = 234;
    anchor_lang::solana_program::program::invoke(
        &system_instruction::create_account(
            payer.key,
            &mint.key(),
            Rent::get()?.minimum_balance(extension_len),
            extension_len as u64,
            token_program.key,
        ),
        &[payer.to_account_info(), mint.to_account_info()],
    )?;

    // Initialize the metadata extension in the mint
    let bump = ctx.bumps.metadata_pointer;
    anchor_lang::solana_program::program::invoke_signed(
        &metadata_pointer::instruction::initialize(
            token_program.key,
            mint.key,
            Some(payer.key()),
            Some(metadata_pointer.key()),
        )?,
        &[mint.to_account_info()],
        &[&[
            &mint.key.to_bytes(),
            "token22".as_bytes(),
            "metadata_pointer".as_bytes(),
            &[bump],
        ]],
    )?;

    // Initialize the mint
    anchor_spl::token_interface::initialize_mint2(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token_interface::InitializeMint2 {
                mint: mint.to_account_info(),
            },
        ),
        0,
        payer.key,
        Some(program_authority.key),
    )?;

    // create ATA for the user
    msg!("Writing to ATA");
    associated_token::create(CpiContext::new(
        associated_token_program.to_account_info(),
        {
            associated_token::Create {
                payer: payer.to_account_info(),
                associated_token: ata.to_account_info(),
                mint: mint.to_account_info(),
                authority: payer.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            }
        },
    ))?;

    // mint to the payer's wallet
    msg!("Minting to user's wallet");
    mint_to(
        CpiContext::new(
            token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: ata.to_account_info(),
                authority: payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Freeze account
    let program_auth_bump = ctx.bumps.program_authority;
    anchor_spl::token_2022::freeze_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        FreezeAccount {
            mint: mint.to_account_info(),
            account: ata.to_account_info(),
            authority: program_authority.to_account_info(),
        },
        &[&["AUTHORITY".as_bytes(), &[program_auth_bump]]],
    ))?;

    Ok(())
}
