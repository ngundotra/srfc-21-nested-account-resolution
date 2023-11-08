use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ExternalIAccountMeta {
    pubkey: Pubkey,
    signer: bool,
    writable: bool,
}
