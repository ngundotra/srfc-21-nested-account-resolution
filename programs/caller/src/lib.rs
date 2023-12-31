use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;

pub use processor::return_data::*;
pub use processor::swap::*;
pub use processor::transfer::*;

declare_id!("8dHQbAAjuxANBSjsEdFMF4d5wMfTS3Ro2DTLaawBLvJ3");

#[program]
pub mod caller {
    use super::*;

    pub fn preflight_transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
    ) -> Result<()> {
        processor::transfer::preflight_transfer(ctx)
    }

    pub fn transfer<'info>(ctx: Context<'_, '_, '_, 'info, Transfer<'info>>) -> Result<()> {
        processor::transfer::transfer(ctx)
    }

    pub fn preflight_swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        processor::swap::preflight_swap(ctx)
    }

    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        processor::swap::swap(ctx)
    }

    pub fn return_data<'info>(
        ctx: Context<'_, '_, '_, 'info, Noop<'info>>,
        amount: u32,
    ) -> Result<()> {
        processor::return_data::return_data(ctx, amount)
    }
}
