use std::collections::BTreeMap;

use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{prelude::*, solana_program::program::set_return_data};
use anchor_spl::token_interface::Token2022;
use bytemuck::bytes_of;
use spl_token_2022::{
    extension::{metadata_pointer::MetadataPointer, BaseStateWithExtensions, StateWithExtensions},
    state::Mint,
};

#[derive(Accounts)]
pub struct Describe<'info> {
    /// CHECK:
    pub asset: AccountInfo<'info>,
}

// TODO(ngundotra): Need to consolidate the constraints between
// preflight and the actual instruction. Kind of tricky to get
// right, but I think it's possible.

enum Field {
    AccountOwner,
}

struct RequestAccount {
    account: Pubkey,
    derivation: Field,
}

// Getting this right is really important because it's going to force me
// to use generics w.r.t. the extension types and their constraints
// which require multiple account introspections
pub fn preflight_describe<'info>(ctx: Context<'_, '_, '_, 'info, Describe<'info>>) -> Result<()> {
    let mut requested_accounts = AdditionalAccounts::new();
    let asset = &ctx.accounts.asset;

    if *asset.owner != Token2022::id() {
        msg!("Can only describe token22 tokens right now");
        return Err(ProgramError::InvalidAccountData.into());
    }

    let accounts = &mut ctx.remaining_accounts.into_iter();

    let bytes = asset.try_borrow_data().unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&bytes)?;
    let exts = mint_state.get_extension_types()?;

    let mut constraints: Vec<RequestAccount> = vec![];
    let mut given_accounts = BTreeMap::new();
    for ext in exts {
        if ext == spl_token_2022::extension::ExtensionType::MetadataPointer {
            msg!("Metadata pointer");

            let pointer = mint_state.get_extension::<MetadataPointer>()?;
            if let Some(metadata_address) = Option::<Pubkey>::from(pointer.metadata_address) {
                match next_account_info(accounts) {
                    Ok(account) => {
                        if account.key != &metadata_address {
                            msg!(
                                "Invalid metadata account address. Expecting: {}, found: {}",
                                metadata_address,
                                account.key
                            );
                            return Err(ProgramError::InvalidAccountData.into());
                        }
                        given_accounts.insert(account.key, account);
                    }
                    Err(_) => {
                        requested_accounts.add_account(&metadata_address, false)?;
                        constraints.push(RequestAccount {
                            account: metadata_address,
                            derivation: Field::AccountOwner,
                        })
                    }
                }
            }
        }
    }
    for constraint in constraints {
        match constraint.derivation {
            Field::AccountOwner => {
                let account = given_accounts.get(&constraint.account).unwrap();
                let owner = account.owner;
                match next_account_info(accounts) {
                    Ok(account) => {
                        if account.key != owner {
                            msg!(
                                "Invalid account owner. Expecting: {}, found: {}",
                                owner,
                                account.key
                            );
                            return Err(ProgramError::InvalidAccountData.into());
                        }
                        given_accounts.insert(account.key, account);
                    }
                    Err(_) => {
                        requested_accounts.add_account(owner, false)?;
                    }
                }
            }
        }
    }

    set_return_data(bytes_of(&requested_accounts));
    Ok(())
}

pub fn describe(ctx: Context<Describe>) -> Result<()> {
    let asset = &ctx.accounts.asset;

    if *asset.owner != Token2022::id() {
        msg!("Can only describe token22 tokens right now");
        return Err(ProgramError::InvalidAccountData.into());
    }

    let remaining_accounts = &mut ctx.remaining_accounts.into_iter();

    let bytes = asset.try_borrow_data().unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&bytes)?;
    let pointer_info = mint_state.get_extension::<MetadataPointer>()?;

    if let Some(metadata_address) = Option::<Pubkey>::from(pointer_info.metadata_address) {
        let metadata_pointer_acc = next_account_info(remaining_accounts)?;
        assert!(
            metadata_address == *metadata_pointer_acc.key,
            "Wrong metadata pointer account. Expecting: {}, received: {}",
            metadata_address,
            metadata_pointer_acc.key
        );
        let metadata_program = next_account_info(remaining_accounts)?;
        assert!(
            metadata_program.key == metadata_pointer_acc.owner,
            "Wrong metadata program. Expecting: {}, received: {}",
            metadata_pointer_acc.owner,
            metadata_program.key
        );

        // invoke `emit` from token_metadata on the program
        // then serialize & send back
        // actually I can check if its my own program that owns the metadata, and then deserialize,
        // but in general would be great to do this more generally
    }

    Ok(())
}
