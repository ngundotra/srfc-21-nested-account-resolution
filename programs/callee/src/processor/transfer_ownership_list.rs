use crate::state::OwnershipList;
use additional_accounts_request::{AdditionalAccounts, IAccountMeta};
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
    page: u8,
) -> Result<()> {
    let ownership_list = &ctx.accounts.ownership_list;
    let mut additional_accounts = AdditionalAccounts {
        accounts: ownership_list
            .accounts
            .iter()
            .map(|key| IAccountMeta {
                pubkey: *key,
                signer: false,
                writable: false,
            })
            .collect(),
        has_more: false,
    };
    additional_accounts.page_to(page)?;
    let return_data = additional_accounts.try_to_vec().unwrap();
    msg!("additional_accounts serialized size: {}", return_data.len());
    set_return_data(&return_data);
    Ok(())
}
