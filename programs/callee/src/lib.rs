use anchor_lang::prelude::*;

pub mod interface;
pub mod processor;
pub mod state;

// Need this to be able to get ExternalIAccountMeta into IDLs
// Please remove once ExternalIAccountMeta is a normal type in Anchor
use processor::add_keypair_node::*;
use processor::add_pda_node::*;
use processor::close_linked_list::*;
use processor::create_linked_list::*;
use processor::create_ownership_list::*;
use processor::init_linked_list_head_node::*;
use processor::return_data::*;
use processor::transfer_linked_list::*;
use processor::transfer_ownership_list::*;

declare_id!("8hKjTVHaCE4U2zMYVx5eu5P9MTCU2imhvZZU31jDnYNA");

/// Example program for building linked lists of pubkeys, transferring them,
/// and closing them. What's unique about this program is that NO SDK is needed
/// to use its instructions. Instead, each one of these program instructions will derive
/// the correct accounts for the instruction based on the user's inputs.
/// The best example of this is in transferLinkedList and closedLinkedList. These instructions
/// modify the whole linked list at once, but only requires 1 head node account to derive the
/// rest of the accounts. More information about how this is done can be found at https://github.com/ngundotra/srfc-21-nested-account-resolution
#[program]
pub mod callee {
    use super::*;

    /// Create a linked list by creating a new node.
    /// You can add child nodes recursively.
    pub fn init_linked_list_head_node<'info>(
        ctx: Context<'_, '_, '_, 'info, InitLinkedListHeadNode<'info>>,
    ) -> Result<()> {
        processor::init_linked_list_head_node::init_linked_list_head_node(ctx)
    }

    pub fn preflight_init_linked_list_head_node<'info>(
        ctx: Context<'_, '_, '_, 'info, InitLinkedListHeadNodeReadonly<'info>>,
    ) -> Result<()> {
        processor::init_linked_list_head_node::preflight_init_linked_list_head_node(ctx)
    }

    /// This method allows you to add a node to previously created node.
    /// The child node will be a PDA derived from the parent node's address.
    pub fn add_pda_node<'info>(ctx: Context<'_, '_, '_, 'info, AddPdaNode<'info>>) -> Result<()> {
        processor::add_pda_node::add_pda_node(ctx)
    }

    pub fn preflight_add_pda_node<'info>(
        ctx: Context<'_, '_, '_, 'info, AddPdaNodeReadonly<'info>>,
    ) -> Result<()> {
        processor::add_pda_node::preflight_add_pda_node(ctx)
    }

    /// This method allows you to add a node to a previously created node, using a new keypair generated by the user.
    pub fn add_keypair_node<'info>(
        ctx: Context<'_, '_, '_, 'info, AddKeypairNode<'info>>,
    ) -> Result<()> {
        processor::add_keypair_node::add_keypair_node(ctx)
    }

    pub fn preflight_add_keypair_node<'info>(
        ctx: Context<'_, '_, '_, 'info, AddKeypairNodeReadonly<'info>>,
    ) -> Result<()> {
        processor::add_keypair_node::preflight_add_keypair_node(ctx)
    }

    /// This method allows you transfer the current node and all of its children
    /// to another account's ownership. Please note that if the current node
    /// has parent nodes, those will not be transferred in this call. You must
    /// also call this method on the current node's parent if you wish to transfer the parent.
    pub fn transfer_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_linked_list::transfer_linked_list(ctx, destination)
    }

    pub fn preflight_transfer_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferLinkedList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_linked_list::preflight_transfer_linked_list(ctx, destination)
    }

    /// This method will close the current node and all of its child nodes
    /// and return the lamports for rent back to the owner.
    pub fn close_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseLinkedList<'info>>,
    ) -> Result<()> {
        processor::close_linked_list::close_linked_list(ctx)
    }

    pub fn preflight_close_linked_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseLinkedList<'info>>,
    ) -> Result<()> {
        processor::close_linked_list::preflight_close_linked_list(ctx)
    }

    /// This method allows you to transfer an ownership list to another account's ownership.
    pub fn transfer_ownership_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_ownership_list::transfer_ownership_list(ctx, destination)
    }

    pub fn preflight_transfer_ownership_list<'info>(
        ctx: Context<'_, '_, 'info, 'info, TransferOwnershipList<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        processor::transfer_ownership_list::preflight_transfer_ownership_list(ctx, destination)
    }

    /// Boilerplate initialization methods
    /// Test account data introspection
    pub fn create_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateLinkedList<'info>>,
        num: u32,
    ) -> Result<()> {
        processor::create_linked_list::create_linked_list(ctx, num)
    }

    /// Creates an ownership list.
    /// An ownership list is an account that stores a list of other pubkeys
    /// that it controls. An ownership list has one authority that can
    /// transfer it. This is used for internal testing of account resolution strategies.
    pub fn create_ownership_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateOwnershipList<'info>>,
        num: u32,
    ) -> Result<()> {
        processor::create_ownership_list::create_ownership_list(ctx, num)
    }

    /// Utility instruction used for benchmarking compute unit costs of using return data
    pub fn return_data<'info>(ctx: Context<'_, '_, 'info, 'info, Noop>, amount: u32) -> Result<()> {
        processor::return_data::return_data(ctx, amount)
    }
}
