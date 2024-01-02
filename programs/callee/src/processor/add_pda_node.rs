use crate::state::Node;
use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use anchor_lang::solana_program::system_program;

/// This is just to make it easy to test via the explorer
#[derive(Accounts)]
pub struct AddPdaNode<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut)]
    parent_node: Account<'info, Node>,
    #[account(init, payer=payer, space=8 + std::mem::size_of::<Node>(), seeds=[&parent_node.key().to_bytes(), "linked_list".as_bytes()], bump)]
    new_node: Account<'info, Node>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddPdaNodeReadonly<'info> {
    payer: Signer<'info>,
    parent_node: Account<'info, Node>,
}

pub fn preflight_add_pda_node<'info>(
    ctx: Context<'_, '_, '_, 'info, AddPdaNodeReadonly<'info>>,
) -> Result<()> {
    let parent_node = &ctx.accounts.parent_node;
    let mut accounts = AdditionalAccounts::new();
    let pda_node = Pubkey::find_program_address(
        &[&parent_node.key().to_bytes(), "linked_list".as_bytes()],
        &crate::id(),
    )
    .0;
    accounts.add_account(&pda_node, true)?;
    accounts.add_account(&system_program::id(), false)?;
    set_return_data(bytemuck::bytes_of(&accounts));
    Ok(())
}

pub fn add_pda_node<'info>(ctx: Context<'_, '_, '_, 'info, AddPdaNode<'info>>) -> Result<()> {
    let parent_node = &mut ctx.accounts.parent_node;
    let new_node = &mut ctx.accounts.new_node;
    new_node.owner = ctx.accounts.payer.key();
    parent_node.next = Some(new_node.key());
    Ok(())
}
