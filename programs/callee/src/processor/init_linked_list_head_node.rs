use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use anchor_lang::solana_program::system_program;

/// This is just to make it easy to test via the explorer
#[derive(Accounts)]
pub struct InitLinkedListHeadNode<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(init, payer=payer, space=8 + std::mem::size_of::<Node>())]
    node: Account<'info, Node>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitLinkedListHeadNodeReadonly<'info> {
    payer: Signer<'info>,
    /// CHECK:
    node: UncheckedAccount<'info>,
}

pub fn preflight_init_linked_list_head_node<'info>(
    ctx: Context<'_, '_, '_, 'info, InitLinkedListHeadNodeReadonly<'info>>,
) -> Result<()> {
    let mut accounts = AdditionalAccounts::new();
    accounts.add_account(&system_program::id(), false)?;
    set_return_data(bytemuck::bytes_of(&accounts));
    Ok(())
}

pub fn init_linked_list_head_node<'info>(
    ctx: Context<'_, '_, '_, 'info, InitLinkedListHeadNode<'info>>,
) -> Result<()> {
    ctx.accounts.node.owner = ctx.accounts.payer.key();
    Ok(())
}
