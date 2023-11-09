use anchor_lang::prelude::*;

/// This is what other crates should use
#[derive(Accounts)]
pub struct ITransferLinkedList<'info> {
    /// CHECK:
    pub owner: AccountInfo<'info>,
    /// CHECK:
    pub head_node: AccountInfo<'info>,
}
