use anchor_lang::prelude::*;
pub mod error;
pub mod processor;
pub mod state;

use processor::initialize::*;
use state::fanout::MembershipModel;

declare_id!("CJAFX8XgZnTNPbVRUFBYda2b43KzzL7cyVVWFJD9rBby");

#[program]
pub mod hydra_generic {
    use super::*;

    pub fn preflight_initialize(
        ctx: Context<InitializeFanoutMsaReadonly>,
        args: InitializeFanoutArgs,
        model: MembershipModel,
    ) -> Result<()> {
        processor::initialize::preflight_initialize(ctx, args, model)
    }

    pub fn initialize(
        ctx: Context<InitializeFanout>,
        args: InitializeFanoutArgs,
        model: MembershipModel,
    ) -> Result<()> {
        processor::initialize::initialize(ctx, args, model)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
