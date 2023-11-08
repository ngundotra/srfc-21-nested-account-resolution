use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;
pub mod state;

// Need this to be able to get ExternalIAccountMeta into IDLs
// Please remove once ExternalIAccountMeta is a normal type in Anchor
pub use interface::meta::*;
use processor::create_linked_list::*;
use processor::transfer_linked_list::*;

declare_id!("8hKjTVHaCE4U2zMYVx5eu5P9MTCU2imhvZZU31jDnYNA");

#[program]
pub mod benchmark_aar_callee {
    use super::*;

    /// Transfers all nodes in a linked list to a different owner
    pub fn transfer_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferNested<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_linked_list::transfer_linked_list(ctx, destination)
    }

    pub fn preflight_transfer_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferNested<'info>>,
        destination: Pubkey,
        page: u8,
    ) -> Result<()> {
        processor::transfer_linked_list::preflight_transfer_linked_list(ctx, destination, page)
    }

    pub fn create_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateLinkedList<'info>>,
        num: u32,
    ) -> Result<()> {
        processor::create_linked_list::create_linked_list(ctx, num)
    }
}
