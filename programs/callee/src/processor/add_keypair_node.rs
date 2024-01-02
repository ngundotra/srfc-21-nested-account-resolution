use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use anchor_lang::solana_program::system_program;

/// This is just to make it easy to test via the explorer
#[derive(Accounts)]
pub struct AddKeypairNode<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut)]
    parent_node: Account<'info, Node>,
    #[account(init, payer=payer, space=8 + std::mem::size_of::<Node>())]
    new_node: Account<'info, Node>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddKeypairNodeReadonly<'info> {
    payer: Signer<'info>,
    parent_node: Account<'info, Node>,
    /// CHECK: null
    new_node: UncheckedAccount<'info>,
}

pub fn preflight_add_keypair_node<'info>(
    ctx: Context<'_, '_, '_, 'info, AddKeypairNodeReadonly<'info>>,
) -> Result<()> {
    let mut accounts = AdditionalAccounts::new();
    accounts.add_account(&system_program::id(), false)?;
    set_return_data(bytemuck::bytes_of(&accounts));
    Ok(())
}

pub fn add_keypair_node<'info>(
    ctx: Context<'_, '_, '_, 'info, AddKeypairNode<'info>>,
) -> Result<()> {
    let parent_node = &mut ctx.accounts.parent_node;
    let new_node = &mut ctx.accounts.new_node;
    new_node.owner = ctx.accounts.payer.key();
    parent_node.next = Some(new_node.key());
    Ok(())
}
