use crate::state::OwnershipList;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{prelude::*, solana_program::program::set_return_data};

#[derive(Accounts)]
pub struct TransferOwnershipList<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub ownership_list: Account<'info, OwnershipList>,
}

pub fn transfer_ownership_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
    destination: Pubkey,
) -> Result<()> {
    let ownership_list = &mut ctx.accounts.ownership_list;
    ownership_list.owner = destination;

    let remaining_accounts = &mut ctx.remaining_accounts.into_iter();
    for account in ownership_list.accounts.iter() {
        let given_acc = next_account_info(remaining_accounts)?;
        if given_acc.key != account {
            msg!(
                "Invalid account {}, was expecting: {}",
                given_acc.key,
                account
            );
            return Err(ProgramError::InvalidInstructionData.into());
        }
    }
    Ok(())
}

pub fn preflight_transfer_ownership_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
    destination: Pubkey,
) -> Result<()> {
    let ownership_list = &ctx.accounts.ownership_list;

    let mut additional_accounts = AdditionalAccounts::new();
    let mut accounts_iter = ctx.remaining_accounts.into_iter();

    // Find which accounts have already been added
    let mut insert_index: usize = 0;
    for account_key in ownership_list.accounts.iter() {
        match next_account_info(&mut accounts_iter) {
            Ok(acc) => {
                if acc.key != account_key {
                    break;
                } else {
                    insert_index += 1
                }
            }
            Err(..) => {
                break;
            }
        }
    }

    for account_key in ownership_list.accounts[insert_index..].iter() {
        if additional_accounts.has_space_available() {
            additional_accounts.add_account(account_key, false).unwrap();
        } else {
            additional_accounts.set_has_more(true)
        }
    }

    // Logging
    msg!(
        "additional_accounts serialized size: {}",
        additional_accounts.num_accounts
    );
    let bytes = bytemuck::bytes_of(&additional_accounts);
    msg!("additional_accounts serialized: {}", bytes.len());
    set_return_data(bytes);
    Ok(())
}
