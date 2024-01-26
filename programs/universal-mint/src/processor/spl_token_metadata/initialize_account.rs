use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Token2022Initialize<'info> {
    /// CHECK:
    pub metadata: AccountInfo<'info>,
    /// CHECK:
    pub update_authority: AccountInfo<'info>,
    /// CHECK:
    pub mint: AccountInfo<'info>,
    /// CHECK:
    pub mint_authority: AccountInfo<'info>,
}

pub fn t22_initialize<'info>(
    ctx: Context<'_, '_, '_, 'info, Token2022Initialize<'info>>,
) -> Result<()> {
    msg!("Not implemented. Please use create_spl_token_extension_metadata");
    Ok(())
}
