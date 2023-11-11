use crate::state::Node;
use additional_accounts_request::{AdditionalAccounts, IAccountMeta};
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
    page: u8,
) -> Result<()> {
    let mut accounts: Vec<IAccountMeta> = vec![];
    let mut has_more: bool = false;

    ctx.remaining_accounts.iter().for_each(|account| {
        msg!("> received: {}", account.key);
    });
    let mut accounts_iter = ctx.remaining_accounts.into_iter();

    let mut current_node = ctx.accounts.head_node.to_owned();
    while current_node.next.is_some() {
        let next_node = current_node.next.unwrap();
        accounts.push(IAccountMeta {
            pubkey: next_node,
            signer: false,
            writable: true,
        });
        match next_account_info(&mut accounts_iter) {
            Ok(acct) => {
                if acct.key() != next_node {
                    msg!("Missing: {}", next_node.to_string());
                    has_more = true;
                    break;
                } else {
                    current_node = Account::<Node>::try_from_unchecked(&acct)?;
                }
            }
            _ => {
                msg!("Missing: {}", next_node.to_string());
                has_more = true;
                break;
            }
        }
    }

    msg!("callee requested accounts: {}", accounts.len());
    if accounts.len() >= 1 {
        msg!(
            "callee last account: {}",
            accounts[accounts.len() - 1].pubkey.to_string()
        );
        msg!("callee has_more: {}", has_more);
    }

    let mut additional_accounts = AdditionalAccounts { accounts, has_more };
    additional_accounts.page_to(page)?;
    set_return_data(&additional_accounts.try_to_vec()?);
    Ok(())
}
