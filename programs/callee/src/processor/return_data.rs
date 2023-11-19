use anchor_lang::{prelude::*, solana_program::program::set_return_data};

#[derive(Accounts)]
pub struct Noop {}

pub fn return_data<'info>(ctx: Context<'_, '_, 'info, 'info, Noop>, amount: u32) -> Result<()> {
    let data: &[u8];
    if amount == 512 {
        data = &[0u8; 512]
    } else if amount == 1024 {
        data = &[0u8; 1024];
    } else {
        data = &[0u8];
    }
    set_return_data(&data);
    Ok(())
}
