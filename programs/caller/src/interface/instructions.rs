//! This is what other crates should use
use additional_accounts_request::InterfaceInstruction;
use anchor_lang::prelude::*;

/// Use this to transfer anything owned by any program that conforms to `ITransfer`
/// Including transferring a linked list of accounts defined by a LinkedList program
/// to a new destination
#[derive(Accounts)]
pub struct ITransferAnything<'info> {
    /// CHECK:
    pub program: AccountInfo<'info>,
    /// CHECK:
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub object: AccountInfo<'info>,
    /// CHECK:
    pub destination: AccountInfo<'info>,
}

impl<'info> InterfaceInstruction for ITransferAnything<'info> {
    fn instruction_name() -> String {
        "transfer".to_string()
    }
}
