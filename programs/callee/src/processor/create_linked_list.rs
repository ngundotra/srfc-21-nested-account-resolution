use crate::state::Node;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::Discriminator;

// Boilerplate to support calling `transfer_linked_list`
// This writes accounts to a `node` account, which contains pointer to next node
//
// This is great for testing Account-Data introspection with paging, but not so great for
// testing the max number of accounts I can get with nested-account-resolution.
// The reason is because creating a linked list requires a lot of keypair signatures lol
#[derive(Accounts)]
pub struct CreateLinkedList<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn create_linked_list<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateLinkedList<'info>>,
    num: u32,
) -> Result<()> {
    let mut accounts_iter = ctx.remaining_accounts.into_iter();
    let mut prev_node: Option<Node> = None;
    let mut prev_ai: Option<&AccountInfo> = None;

    let payer = ctx.accounts.payer.to_account_info();
    for i in 0..num {
        let acct = next_account_info(&mut accounts_iter)?;

        let space: u64 = 8 + std::mem::size_of::<Node>() as u64;
        let lamports = Rent::get()?.minimum_balance(space as usize);
        let ix = system_instruction::create_account(
            ctx.accounts.payer.key,
            acct.key,
            lamports,
            space,
            &crate::id(),
        );
        invoke(&ix, &[payer.clone(), acct.clone()])?;

        let node = Node {
            id: i,
            next: None,
            owner: payer.key(),
        };

        if let Some(mut prev_node) = prev_node {
            prev_node.next = Some(acct.key());
            let mut data = Node::discriminator().to_vec();
            data.extend_from_slice(&prev_node.try_to_vec()?);

            let mut account_data = prev_ai.unwrap().try_borrow_mut_data()?;
            account_data[0..data.len()].copy_from_slice(&data);
        }
        prev_node = Some(node);
        prev_ai = Some(acct);
    }

    if let Some(mut prev_node) = prev_node {
        prev_node.next = None;
        let mut data = Node::discriminator().to_vec();
        data.extend_from_slice(&prev_node.try_to_vec()?);

        let mut account_data = prev_ai.unwrap().try_borrow_mut_data()?;
        account_data[0..data.len()].copy_from_slice(&data);
    }
    Ok(())
}
