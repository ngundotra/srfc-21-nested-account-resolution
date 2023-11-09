//! This is what other crates should use
use additional_accounts_request::InterfaceInstruction;
use anchor_lang::prelude::*;

// This is what we need to use the transfer linked list
#[derive(Accounts)]
pub struct ITransferLinkedList<'info> {
    /// CHECK:
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub head_node: AccountInfo<'info>,
}

impl<'info> InterfaceInstruction for ITransferLinkedList<'info> {
    fn instruction_name() -> String {
        "transfer_linked_list".to_string()
    }
}
