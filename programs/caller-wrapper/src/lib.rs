use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;

pub use interface::meta::*;
pub use processor::transfer::*;

declare_id!("BoU7xvB9ZUrSxpRsYaeKbjj5Xv7MdR2YiSRgMgwoij6k");

#[program]
pub mod caller_wrapper {
    use super::*;

    pub fn preflight_transfer<'info>(
        ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>,
        page: u8,
    ) -> Result<()> {
        processor::transfer::preflight_transfer(ctx, page)
    }

    pub fn transfer<'info>(ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>) -> Result<()> {
        processor::transfer::transfer(ctx)
    }
}
