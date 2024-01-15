use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{prelude::*, solana_program::program::set_return_data};

#[derive(Accounts)]
pub struct TransferLinkedList<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub head_node: Account<'info, Node>,
}

pub fn transfer_linked_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
    destination: Pubkey,
) -> Result<()> {
    let current_node = &mut ctx.accounts.head_node;
    msg!("current: {:?}", &current_node.owner);
    current_node.owner = destination;
    current_node.exit(&crate::id())?;

    let mut current_node = current_node.clone().into_inner();

    let mut accounts_iter = ctx.remaining_accounts.into_iter();
    while current_node.next.is_some() {
        let next_node = current_node.next.unwrap();
        let next_acct = next_account_info(&mut accounts_iter)?;

        if next_acct.key() != next_node {
            msg!(
                "Invalid account {}, was expecting: {}",
                next_acct.key(),
                next_node
            );
            return Err(ProgramError::InvalidInstructionData.into());
        }

        let mut next_node_acct = Account::<Node>::try_from(next_acct)?;
        next_node_acct.owner = destination;
        next_node_acct.exit(&crate::id())?;

        current_node = next_node_acct.clone().into_inner();
    }

    Ok(())
}

pub fn preflight_transfer_linked_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
    destination: Pubkey,
) -> Result<()> {
    ctx.remaining_accounts.iter().for_each(|account| {
        msg!("> received: {}", account.key);
    });
    let mut accounts_iter = ctx.remaining_accounts.into_iter();

    let mut additional_accounts = AdditionalAccounts::new();
    let mut current_node = ctx.accounts.head_node.to_owned();
    while current_node.next.is_some() && additional_accounts.has_space_available() {
        let next_node = current_node.next.unwrap();
        match next_account_info(&mut accounts_iter) {
            Ok(acct) => {
                if acct.key() != next_node {
                    msg!("Missing: {}", next_node.to_string());
                    additional_accounts.add_account(&next_node, true)?;
                    additional_accounts.set_has_more(true);
                    break;
                } else {
                    current_node = Account::<Node>::try_from_unchecked(&acct)?;
                }
            }
            _ => {
                msg!("Missing: {}", next_node.to_string());
                additional_accounts.add_account(&next_node, true)?;
                additional_accounts.set_has_more(true);
                break;
            }
        }
    }

    msg!(
        "callee requested accounts: {}",
        additional_accounts.num_accounts
    );
    if additional_accounts.num_accounts >= 1 {
        msg!(
            "callee last account: {}",
            additional_accounts.accounts[additional_accounts.num_accounts as usize - 1].to_string()
        );
        msg!("callee has_more: {}", additional_accounts.has_more);
    }

    set_return_data(bytemuck::bytes_of(&additional_accounts));
    Ok(())
}
