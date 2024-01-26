use additional_accounts_request::AdditionalAccounts;
use anchor_lang::prelude::*;

use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
use bytemuck::bytes_of;
use solana_program::program::set_return_data;

mod program_keys;
pub use program_keys::*;

mod processor;
use processor::*;

declare_id!("8BMnVbSD8L9gbe5Qw6jSKXPjmYM2c7wa4h9rGLPXBaJw");

#[program]
pub mod libreplex_manager {
    use super::*;

    pub fn preflight_initialize(
        ctx: Context<InitializeReadonly>,
        args: InitialiseInputV2,
    ) -> Result<()> {
        processor::preflight_initialize(ctx, args)
    }

    pub fn initialize(ctx: Context<Initialize>, args: InitialiseInputV2) -> Result<()> {
        processor::initialize(ctx, args)
    }

    pub fn preflight_deploy(ctx: Context<DeployLegacyV2CtxReadonly>) -> Result<()> {
        processor::preflight_deploy(ctx)
    }

    pub fn deploy(ctx: Context<DeployLegacyV2Ctx>) -> Result<()> {
        processor::deploy(ctx)
    }
}
