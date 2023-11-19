use anchor_lang::{prelude::*, solana_program::program::invoke};

#[derive(Accounts)]
pub struct Noop<'info> {
    /// CHECK:
    program: AccountInfo<'info>,
}

pub fn return_data<'info>(ctx: Context<'_, '_, '_, 'info, Noop<'info>>, amount: u32) -> Result<()> {
    let mut ix_data: Vec<u8> =
        anchor_lang::solana_program::hash::hash(format!("global:return_data").as_bytes())
            .to_bytes()[..8]
            .to_vec();
    ix_data.extend(amount.to_le_bytes());
    invoke(
        &anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.program.key(),
            accounts: vec![],
            data: ix_data,
        },
        &[],
    )?;
    Ok(())
}
