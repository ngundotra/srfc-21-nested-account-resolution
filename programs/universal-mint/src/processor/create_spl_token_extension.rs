use anchor_lang::prelude::*;

use anchor_lang::solana_program::system_instruction;
use anchor_spl::{token::Mint, token_interface::Token2022};

#[derive(Accounts)]
pub struct CreateSplToken22<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut)]
    mint: Signer<'info>,
    token_program: Program<'info, Token2022>,
    system_program: Program<'info, System>,
}

/// Technically, this is incomplete
pub fn create_spl_token_extension(ctx: Context<CreateSplToken22>, decimals: u8) -> Result<()> {
    anchor_lang::solana_program::program::invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            &ctx.accounts.mint.key(),
            Rent::get()?.minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            ctx.accounts.token_program.key,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    anchor_spl::token_interface::initialize_mint2(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        decimals,
        ctx.accounts.payer.key,
        Some(ctx.accounts.payer.key),
    )?;
    Ok(())
}
