use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{prelude::*, solana_program::program_pack::Pack};

use anchor_lang::solana_program::program::set_return_data;
use anchor_spl::associated_token::{AssociatedToken, Create};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_2022::spl_token_2022::extension::StateWithExtensions,
    token_2022::spl_token_2022::state::Account as SplTokenAccount,
    token_2022::spl_token_2022::state::Mint as SplMintAccount, token_2022::Token2022,
    token_interface::TransferChecked,
};
use bytemuck::bytes_of;

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub asset: AccountInfo<'info>,
    /// CHECK:
    pub destination: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferTokenReadonly<'info> {
    /// CHECK:
    pub owner: AccountInfo<'info>,
    /// CHECK:
    pub asset: AccountInfo<'info>,
    /// CHECK:
    pub destination: AccountInfo<'info>,
}

pub fn preflight_transfer_token<'info>(
    ctx: Context<'_, '_, '_, 'info, TransferTokenReadonly<'info>>,
    amount: u64,
) -> Result<()> {
    let owner = &ctx.accounts.owner;
    let destination = &ctx.accounts.destination;
    let asset = &ctx.accounts.asset;

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    msg!("Humbug!");
    if *asset.owner == Token2022::id() {
        msg!("Preflighting transfer token22");
        preflight_transfer_token_2022(&owner, &asset, &destination, &mut remaining_accounts.iter())
    } else {
        msg!("Can only transfer token22 tokens right now");
        Err(ProgramError::InvalidAccountData.into())
    }
}

pub fn preflight_transfer_token_2022<'info>(
    owner: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    accounts: &mut core::slice::Iter<AccountInfo<'info>>,
) -> Result<()> {
    let mut requested_accounts = AdditionalAccounts::new();

    let dest_ata =
        get_associated_token_address_with_program_id(destination.key, asset.key, &Token2022::id());
    let to_check = [
        // owner's ata
        (
            get_associated_token_address_with_program_id(owner.key, asset.key, &Token2022::id()),
            true,
        ),
        // destination's ata
        (dest_ata, true),
        // token program
        (Token2022::id(), false),
    ];

    let mut dest_ata_exists = false;
    let mut last_idx = 0;
    for (idx, account) in accounts.enumerate() {
        if idx >= to_check.len() {
            break;
        }

        let (expected, expected_writability) = to_check.get(idx).unwrap();
        if account.key != expected {
            msg!("Expected {}, recieved: {}", expected, account.key);
            return Err(ProgramError::InvalidAccountData.into());
        }
        if account.is_writable != *expected_writability {
            msg!(
                "Expected account {} mutability to be {} recieved: {}",
                account.key,
                expected_writability,
                account.is_writable
            );
            return Err(ProgramError::InvalidAccountData.into());
        }
        if *account.key == dest_ata {
            dest_ata_exists = account.try_borrow_data()?.len() > 0;
        }
        last_idx = idx;
    }

    if last_idx < to_check.len() {
        for idx in last_idx..to_check.len() {
            let (expected, expected_writability) = to_check.get(idx).unwrap();
            requested_accounts.add_account(expected, *expected_writability)?;
        }

        requested_accounts.set_has_more(true);
    }

    // If the destination ATA does not exist yet, then we will have to create it for them
    if !dest_ata_exists {
        requested_accounts.add_account(&AssociatedToken::id(), false)?;
        requested_accounts.add_account(&System::id(), false)?;
        requested_accounts.set_has_more(false);
    }

    set_return_data(bytes_of(&requested_accounts));
    Ok(())
}

pub fn transfer_token<'info>(
    ctx: Context<'_, '_, '_, 'info, TransferToken<'info>>,
    amount: u64,
) -> Result<()> {
    let owner = &ctx.accounts.owner;
    let destination = &ctx.accounts.destination;
    let asset = &ctx.accounts.asset;
    let accounts = ctx.remaining_accounts.to_vec();

    if *asset.owner == Token2022::id() {
        transfer_token_2022(
            &owner.to_account_info(),
            &asset,
            &destination,
            &mut accounts.iter(),
            amount,
        )
    } else {
        msg!("Can only transfer token22 tokens right now");
        Err(ProgramError::InvalidAccountData.into())
    }
}

fn transfer_token_2022<'info>(
    owner: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    accounts: &mut core::slice::Iter<AccountInfo<'info>>,
    amount: u64,
) -> Result<()> {
    let mut remaining_accounts = accounts;

    // We deserialize in closure to make sure we drop the bytes after borrowing
    let decimals: u8 = {
        msg!("Unpacking mint account");
        let bytes = asset.try_borrow_data()?;
        let mint = StateWithExtensions::<SplMintAccount>::unpack(&bytes)?;
        mint.base.decimals
    };

    let source_ata_ai = next_account_info(&mut remaining_accounts)?;
    {
        let bytes = source_ata_ai.try_borrow_data()?;
        // We don't need StateWithExtensionsMut because we're only reading the data, not writing it
        let source_ata = StateWithExtensions::<SplTokenAccount>::unpack(&bytes)?;
        assert!(
            source_ata.base.mint == asset.key(),
            "Malformed accounts. Asset does not match source ata"
        );
        assert!(
            source_ata.base.amount >= amount,
            "Owner does not have enough funds to transfer"
        );
        assert!(
            source_ata.base.owner == owner.key(),
            "Malformed accounts. Owner does not own source ata"
        );
    }

    // We don't unpack the destination_ata because it may not exist yet
    let destination_ata_ai = next_account_info(&mut remaining_accounts)?;
    {
        let expected_destination_ata = get_associated_token_address_with_program_id(
            destination.key,
            asset.key,
            &Token2022::id(),
        );
        assert!(
            destination_ata_ai.key() == expected_destination_ata,
            "Malformed accounts. Destination ata expected: {}, recieved: {}",
            expected_destination_ata,
            &destination_ata_ai.key(),
        );
    }

    let token_program = next_account_info(&mut remaining_accounts)?;
    if token_program.key() != Token2022::id() {
        msg!(
            "Invalid token program. Expected token22, received: {}",
            token_program.key()
        );
        return Err(ProgramError::InvalidAccountData.into());
    }

    // Check if destination ATA exists. If it doesn't then we need to create it
    if destination_ata_ai.data_is_empty() {
        msg!("Destination ATA does not exist. Creating it");
        let associated_token_program = next_account_info(&mut remaining_accounts)?;
        let system_program = next_account_info(&mut remaining_accounts)?;
        anchor_spl::associated_token::create(CpiContext::new(
            associated_token_program.clone(),
            Create {
                payer: owner.clone(),
                authority: destination.clone(),
                mint: asset.to_account_info(),
                token_program: token_program.clone(),
                associated_token: destination_ata_ai.clone(),
                system_program: system_program.clone(),
            },
        ))?;
    }

    anchor_spl::token_2022::transfer_checked(
        CpiContext::new(
            token_program.clone(),
            TransferChecked {
                from: source_ata_ai.clone(),
                mint: asset.to_account_info(),
                to: destination_ata_ai.clone(),
                authority: owner.to_account_info(),
            },
        ),
        amount,
        decimals,
    )?;
    Ok(())
}
