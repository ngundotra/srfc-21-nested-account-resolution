use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::{accounts::account_info, prelude::*, solana_program::program::set_return_data};

pub fn close<'info>(
    info: &AccountInfo<'info>,
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
    msg!("Owner lamports start: {}", owner.lamports());

    let current_node = &mut ctx.accounts.head_node;

    let mut current_node = current_node.clone().into_inner();

    let mut accounts_iter = ctx.remaining_accounts.into_iter();
    while current_node.next.is_some() {
        let expected_value = current_node.next.unwrap();
        let current_ai = next_account_info(&mut accounts_iter)?;

        if *current_ai.key != expected_value {
            msg!(
                "Invalid account {}, was expecting: {}",
                current_ai.key,
                expected_value
            );
            return Err(ProgramError::InvalidInstructionData.into());
        }

        current_node = Account::<Node>::try_from(current_ai)?.into_inner();
        close(current_ai, &mut owner)?;
    }
    msg!("Owner lamports final: {}", owner.lamports());

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
