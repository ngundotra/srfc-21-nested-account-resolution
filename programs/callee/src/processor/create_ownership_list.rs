use crate::state::OwnershipList;
use anchor_lang::prelude::*;

// Boilerplate to test how many possible accounts can I resolve with nested-account-resolution and paging
#[derive(Accounts)]
#[instruction(num: u32)]
pub struct CreateOwnershipList<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    /// Must be keypair
    #[account(init, payer=payer, space=8 + 4 + 32 + 32 * num as usize)]
    ownership_list: Account<'info, OwnershipList>,
    system_program: Program<'info, System>,
}

pub fn create_ownership_list<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateOwnershipList<'info>>,
    num: u32,
) -> Result<()> {
    let ownership_list = &mut ctx.accounts.ownership_list;
    ownership_list.owner = ctx.accounts.payer.key();
    let base = ownership_list.key().to_bytes();
    for i in 0..num {
        let key = Pubkey::find_program_address(&[&base, &i.to_le_bytes()], &crate::id()).0;
        ownership_list.accounts.push(key);
    }
    Ok(())
}
