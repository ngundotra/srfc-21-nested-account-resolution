use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;
pub mod state;

// Need this to be able to get ExternalIAccountMeta into IDLs
// Please remove once ExternalIAccountMeta is a normal type in Anchor
pub use interface::meta::*;
use processor::create_linked_list::*;
use processor::create_ownership_list::*;
use processor::transfer_linked_list::*;
use processor::transfer_ownership_list::*;

declare_id!("8hKjTVHaCE4U2zMYVx5eu5P9MTCU2imhvZZU31jDnYNA");

#[program]
pub mod callee {
    use super::*;

    /// Transfers all nodes in a linked list to a different owner
    pub fn transfer_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_linked_list::transfer_linked_list(ctx, destination)
    }

    pub fn preflight_transfer_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
        destination: Pubkey,
        page: u8,
    ) -> Result<()> {
        processor::transfer_linked_list::preflight_transfer_linked_list(ctx, destination, page)
    }

    pub fn transfer_ownership_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_ownership_list::transfer_ownership_list(ctx, destination)
    }

    pub fn preflight_transfer_ownership_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
        destination: Pubkey,
        page: u8,
    ) -> Result<()> {
        processor::transfer_ownership_list::preflight_transfer_ownership_list(
            ctx,
            destination,
            page,
        )
    }

    /// Boilerplate initialization methods
    /// Test account data introspection
    pub fn create_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateLinkedList<'info>>,
        num: u32,
    ) -> Result<()> {
        processor::create_linked_list::create_linked_list(ctx, num)
    }

    /// Test usefulness of account paging
    pub fn create_ownership_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateOwnershipList<'info>>,
        num: u32,
    ) -> Result<()> {
        processor::create_ownership_list::create_ownership_list(ctx, num)
    }
}
