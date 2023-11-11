use anchor_lang::prelude::*;

#[derive(Debug)]
#[account]
pub struct OwnershipList {
    pub owner: Pubkey,
    pub accounts: Vec<Pubkey>,
}
