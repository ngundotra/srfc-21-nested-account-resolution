use anchor_lang::prelude::*;

#[account]
pub struct MetadataInfo {
    // This field could optionally be derived somehow too
    pub update_authority: Pubkey,
    /// This field needs to be derivable somehow
    pub mint: Pubkey,
    pub name: String,
    pub description: String,
}
