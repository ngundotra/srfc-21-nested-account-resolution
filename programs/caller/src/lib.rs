use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;

pub use interface::meta::*;
pub use processor::transfer::*;

declare_id!("8dHQbAAjuxANBSjsEdFMF4d5wMfTS3Ro2DTLaawBLvJ3");

#[program]
pub mod caller {
    use super::*;

    pub fn preflight_transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
        page: u8,
    ) -> Result<()> {
        processor::transfer::preflight_transfer(ctx, page)
    }

    pub fn transfer<'info>(ctx: Context<'_, '_, '_, 'info, Transfer<'info>>) -> Result<()> {
        processor::transfer::transfer(ctx)
    }
}
