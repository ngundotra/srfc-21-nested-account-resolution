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
    ownerA: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    objectA: AccountInfo<'info>,

    /// CHECK:
    ownerB: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    objectB: AccountInfo<'info>,
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
            }
        });

    match stage {
        0 => {
            let mut additional_accounts = resolve_additional_accounts(
                get_transfer_ix_name(&ctx.accounts.objectA.try_borrow_data()?[0..8])?,
                &CpiContext::new(
                    ctx.accounts.program.clone(),
                    ITransfer {
                        owner: ctx.accounts.ownerA.clone(),
                        object: ctx.accounts.objectA.clone(),
                    },
                )
                .with_remaining_accounts(ctx.remaining_accounts[0..latest_delimiter_idx].to_vec()),
                &get_transfer_args(ctx.accounts.objectA.key),
                false,
            )?;

            // We set set this to true because we have more accounts for our 2nd call
            additional_accounts.set_has_more(true);

            // We can only add delimiter if there is space available.
            // Otherwise we have to wait until another account is requested
            // and then go from there
            if !additional_accounts.has_space_available() {
                set_return_data(bytemuck::bytes_of(&additional_accounts));
                return Ok(());
            }

            //
            if additional_accounts.has_more != 1 {
                additional_accounts.add_account(&get_delimiter(&crate::id()), false)?;
            }

            set_return_data(bytemuck::bytes_of(&additional_accounts));
            Ok(())
        }
        1 => {
            let mut additional_accounts = resolve_additional_accounts(
                get_transfer_ix_name(&ctx.accounts.objectB.try_borrow_data()?[0..8])?,
                &CpiContext::new(
                    ctx.accounts.program.clone(),
                    ITransfer {
                        owner: ctx.accounts.ownerB.clone(),
                        object: ctx.accounts.objectB.clone(),
                    },
                )
                .with_remaining_accounts(ctx.remaining_accounts[latest_delimiter_idx..].to_vec()),
                &get_transfer_args(ctx.accounts.objectB.key),
                false,
            )?;

            if !additional_accounts.has_space_available() {
                set_return_data(bytemuck::bytes_of(&additional_accounts));
                return Ok(());
            }

            if additional_accounts.has_more != 1 {
                additional_accounts.add_account(&get_delimiter(&crate::id()), false)?;
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
            owner: ctx.accounts.ownerA.clone(),
            object: ctx.accounts.objectA.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    let counter = call(
        get_transfer_ix_name(&ctx.accounts.objectA.try_borrow_data()?[0..8])?,
        cpi_ctx,
        ctx.accounts.ownerB.key.try_to_vec().unwrap(),
        0,
        true,
    )?;

    // Second swap leg
    let cpi_ctx = CpiContext::new(
        ctx.accounts.program.clone(),
        ITransfer {
            owner: ctx.accounts.ownerB.clone(),
            object: ctx.accounts.objectB.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    call(
        get_transfer_ix_name(&ctx.accounts.objectB.try_borrow_data()?[0..8])?,
        cpi_ctx,
        ctx.accounts.ownerA.key.try_to_vec().unwrap(),
        counter,
        true,
    )?;
    Ok(())
}
