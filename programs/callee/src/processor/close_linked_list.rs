use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{prelude::*, solana_program::program::set_return_data};

pub fn close<'info>(
    info: AccountInfo<'info>,
    sol_destination: &mut AccountInfo<'info>,
) -> Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    info.assign(&anchor_lang::solana_program::system_program::ID);
    info.realloc(0, false).map_err(Into::into)
}

#[derive(Accounts)]
pub struct CloseLinkedList<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner, close=owner)]
    pub head_node: Account<'info, Node>,
}

pub fn close_linked_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, CloseLinkedList<'info>>,
) -> Result<()> {
    let mut owner = ctx.accounts.owner.to_account_info();

    let current_node = &mut ctx.accounts.head_node;

    let mut current_ai = current_node.to_account_info();
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

        // let derived = Pubkey::find_program_address(
        //     &[&current_key.to_bytes(), "linked_list".as_bytes()],
        //     &crate::id(),
        // )
        // .0;

        close(current_ai, &mut owner)?;

        current_ai = next_acct.clone();
        let next_node_acct = Account::<Node>::try_from(next_acct)?;
        current_node = next_node_acct.clone().into_inner();
    }

    Ok(())
}

pub fn preflight_close_linked_list<'info>(
    ctx: Context<'_, '_, 'info, 'info, CloseLinkedList<'info>>,
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
