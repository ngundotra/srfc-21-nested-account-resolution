use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;

pub use processor::transfer::*;

declare_id!("BoU7xvB9ZUrSxpRsYaeKbjj5Xv7MdR2YiSRgMgwoij6k");

#[program]
pub mod caller_wrapper {
    use super::*;

    pub fn preflight_transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
    ) -> Result<()> {
        processor::transfer::preflight_transfer(ctx)
    }

    pub fn transfer<'info>(ctx: Context<'_, '_, '_, 'info, Transfer<'info>>) -> Result<()> {
        processor::transfer::transfer(ctx)
    }
}
