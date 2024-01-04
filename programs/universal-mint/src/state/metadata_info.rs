use anchor_lang::prelude::*;

#[account]
pub struct MetadataInfo {
    pub name: String,
    pub description: String,
}
