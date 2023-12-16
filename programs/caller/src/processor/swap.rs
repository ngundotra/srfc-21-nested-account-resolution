use additional_accounts_request::{
    call, get_delimiter, identify_additional_accounts, resolve_additional_accounts,
    InterfaceInstruction,
};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::set_return_data},
    Discriminator,
};
use callee::{
    interface::instructions::{ITransfer, ITransferLinkedList, ITransferOwnershipList},
    state::{Node, OwnershipList},
};

use super::transfer;

#[derive(Accounts)]
pub struct Swap<'info> {
    /// CHECK:
    program: AccountInfo<'info>,
    /// CHECK:
    owner_a: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    object_a: AccountInfo<'info>,

    /// CHECK:
    owner_b: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    object_b: AccountInfo<'info>,
}

fn get_transfer_ix_name(account_disc: &[u8]) -> Result<String> {
    let ix_name: String;
    {
        if account_disc == Node::discriminator() {
            ix_name = ITransferLinkedList::instruction_name();
            msg!("linked list");
        } else if account_disc == OwnershipList::discriminator() {
            ix_name = ITransferOwnershipList::instruction_name();
            msg!("ownership list");
        } else {
            msg!("Unknown account discriminator");
            return Err(ProgramError::InvalidAccountData.into());
        }
    }
    Ok(ix_name)
}

fn get_transfer_args(object: &Pubkey) -> Vec<u8> {
    object.try_to_vec().unwrap()
}

pub fn preflight_swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    let delimiter = get_delimiter(&crate::id());
    let mut stage: u8 = 0;
    let mut latest_delimiter_idx = 0;
    ctx.remaining_accounts
        .iter()
        .enumerate()
        .for_each(|(i, acc)| {
            if acc.key() == delimiter {
                stage += 1;
                latest_delimiter_idx = i;
                msg!("Found delimiter at: {}", latest_delimiter_idx);
            }
        });

    msg!(
        "stage: {} | delimiter idx: {} | accs len: {}",
        stage,
        latest_delimiter_idx,
        ctx.remaining_accounts.len()
    );
    match stage {
        0 => {
            let ix_name: String;
            {
                ix_name = get_transfer_ix_name(&ctx.accounts.object_a.try_borrow_data()?[0..8])?;
            }
            let mut additional_accounts = resolve_additional_accounts(
                ix_name,
                &CpiContext::new(
                    ctx.accounts.program.clone(),
                    ITransfer {
                        owner: ctx.accounts.owner_a.clone(),
                        object: ctx.accounts.object_a.clone(),
                    },
                )
                .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
                &get_transfer_args(ctx.accounts.object_a.key),
                false,
            )?;

            // We can only add delimiter if there is space available.
            // Otherwise we have to wait until another account is requested
            // and then go from there
            msg!(
                "has more accounts: {}, has more space: {}, num_accounts: {}",
                additional_accounts.has_more == 1,
                additional_accounts.has_space_available(),
                additional_accounts.num_accounts
            );
            if !additional_accounts.has_space_available() {
                additional_accounts.set_has_more(true);
                set_return_data(bytemuck::bytes_of(&additional_accounts));
                return Ok(());
            }

            // If there are no more accounts returned by first call
            // then we add our delimiter & move on
            if additional_accounts.has_more != 1 {
                additional_accounts.add_account(&get_delimiter(&crate::id()), false)?;
            }
            additional_accounts.set_has_more(true);

            set_return_data(bytemuck::bytes_of(&additional_accounts));
            Ok(())
        }
        1 => {
            let ix_name: String;
            {
                ix_name = get_transfer_ix_name(&ctx.accounts.object_b.try_borrow_data()?[0..8])?;
            }
            let mut additional_accounts = resolve_additional_accounts(
                ix_name,
                &CpiContext::new(
                    ctx.accounts.program.clone(),
                    ITransfer {
                        owner: ctx.accounts.owner_b.clone(),
                        object: ctx.accounts.object_b.clone(),
                    },
                )
                .with_remaining_accounts(ctx.remaining_accounts[latest_delimiter_idx..].to_vec()),
                &get_transfer_args(ctx.accounts.object_b.key),
                false,
            )?;

            if !additional_accounts.has_space_available() {
                set_return_data(bytemuck::bytes_of(&additional_accounts));
                return Ok(());
            }

            if additional_accounts.has_more == 0 {
                additional_accounts.set_has_more(false);
            }

            set_return_data(bytemuck::bytes_of(&additional_accounts));
            Ok(())
        }
        _ => {
            msg!("Too many delimiters passed");
            Err(ProgramError::InvalidInstructionData.into())
        }
    }
}

pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // First swap leg
    let cpi_ctx = CpiContext::new(
        ctx.accounts.program.clone(),
        ITransfer {
            owner: ctx.accounts.owner_a.clone(),
            object: ctx.accounts.object_a.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    let mut ix_name: String;
    {
        ix_name = get_transfer_ix_name(&ctx.accounts.object_a.try_borrow_data()?[0..8])?;
    }
    let delimiter_idx = call(
        ix_name,
        cpi_ctx,
        ctx.accounts.owner_b.key.try_to_vec().unwrap(),
        get_delimiter(&crate::id()),
        0,
        true,
    )?;

    // Second swap leg
    let cpi_ctx = CpiContext::new(
        ctx.accounts.program.clone(),
        ITransfer {
            owner: ctx.accounts.owner_b.clone(),
            object: ctx.accounts.object_b.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    {
        ix_name = get_transfer_ix_name(&ctx.accounts.object_b.try_borrow_data()?[0..8])?;
    }
    call(
        ix_name,
        cpi_ctx,
        ctx.accounts.owner_a.key.try_to_vec().unwrap(),
        get_delimiter(&crate::id()),
        delimiter_idx,
        true,
    )?;
    Ok(())
}
