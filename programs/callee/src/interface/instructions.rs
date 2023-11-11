//! This is what other crates should use
use additional_accounts_request::InterfaceInstruction;
use anchor_lang::prelude::*;

// This is what we need to use the transfer linked list
#[derive(Accounts)]
pub struct ITransfer<'info> {
    /// CHECK:
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub object: AccountInfo<'info>,
}

pub struct ITransferLinkedList {}
impl InterfaceInstruction for ITransferLinkedList {
    fn instruction_name() -> String {
        "transfer_linked_list".to_string()
    }
}

pub struct ITransferOwnershipList {}
impl InterfaceInstruction for ITransferOwnershipList {
    fn instruction_name() -> String {
        "transfer_ownership_list".to_string()
    }
}
