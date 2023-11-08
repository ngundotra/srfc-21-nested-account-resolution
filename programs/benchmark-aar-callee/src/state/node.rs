use anchor_lang::prelude::*;

#[derive(Debug)]
#[account]
pub struct Node {
    pub id: u32,
    pub owner: Pubkey,
    pub next: Option<Pubkey>,
}
