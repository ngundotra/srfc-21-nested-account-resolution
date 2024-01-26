use anchor_lang::solana_program::pubkey::Pubkey;

mod metadata_info;
pub use metadata_info::*;

pub fn get_program_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&"AUTHORITY".as_bytes()], &crate::id())
}
