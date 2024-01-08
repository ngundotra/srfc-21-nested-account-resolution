use crate::{
    error::HydraError,
    state::fanout::{Fanout, MembershipModel},
};
use additional_accounts_request::AdditionalAccounts;
// use anchor_lang::
use anchor_lang::{
    prelude::*,
    solana_program::{program::set_return_data, system_program},
};
use anchor_spl::token::{Mint, Token};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeFanoutArgs {
    pub bump_seed: u8,
    pub native_account_bump_seed: u8,
    pub name: String,
    pub total_shares: u64,
}

// const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
// static ID: Pubkey = pubkey!("My11111111111111111111111111111111111111111");

#[derive(Accounts)]
#[instruction(args: InitializeFanoutArgs)]
pub struct InitializeFanout<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        space = 300,
        seeds = [b"fanout-config", args.name.as_bytes()],
        bump,
        payer = authority
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(mut)]
    pub membership_mint: Account<'info, Mint>,
    #[account(
        init,
        space = 1,
        seeds = [b"fanout-native-account", fanout.key().as_ref()],
        bump,
        payer = authority
    )]
    /// CHECK: check native account
    pub holding_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeFanoutMsaReadonly<'info> {
    pub authority: Signer<'info>,
    /// CHECK:
    pub fanout: UncheckedAccount<'info>,
    pub membership_mint: Account<'info, Mint>,
}

pub fn preflight_initialize(
    ctx: Context<InitializeFanoutMsaReadonly>,
    args: InitializeFanoutArgs,
    model: MembershipModel,
) -> Result<()> {
    let accounts_iter = ctx.remaining_accounts.into_iter();
    let mut additional_accounts = AdditionalAccounts::new();

    // perform the checks here
    let accounts: Vec<(Pubkey, bool)> = vec![
        (
            Pubkey::find_program_address(
                &[b"fanout-native-account", ctx.accounts.fanout.key().as_ref()],
                &crate::id(),
            )
            .0,
            true,
        ),
        (system_program::id(), false),
        (anchor_lang::solana_program::sysvar::rent::id(), false),
    ];

    // you can derive things in order
    // topological sort to get accounts w no derivations
    // level 0 - known immediately {} {} {} {}
    // level 1 - requires derivation {} {}
    // level 2
    // {}

    let mut expected_accounts = accounts.into_iter();
    for acc in accounts_iter {
        match expected_accounts.next() {
            Some(expected_acc) => {
                if *acc.key != expected_acc.0 {
                    msg!("Expected account key: {}", expected_acc.0);
                    return Err(ProgramError::InvalidAccountData.into());
                }
                if acc.is_writable != expected_acc.1 {
                    msg!(
                        "Expected account is_writable to be {}: but is {}",
                        expected_acc.0,
                        acc.is_writable
                    );
                    return Err(ProgramError::InvalidAccountData.into());
                }
            }
            None => return Ok(()),
        };
    }
    for remaining_acc in expected_accounts {
        additional_accounts.add_account(&remaining_acc.0, remaining_acc.1)?;
    }

    let bytes = bytemuck::bytes_of(&additional_accounts);
    set_return_data(&bytes);

    Ok(())
}

pub fn initialize(
    ctx: Context<InitializeFanout>,
    args: InitializeFanoutArgs,
    model: MembershipModel,
) -> Result<()> {
    let membership_mint = &ctx.accounts.membership_mint;
    let fanout = &mut ctx.accounts.fanout;
    fanout.authority = ctx.accounts.authority.to_account_info().key();
    fanout.account_key = ctx.accounts.holding_account.to_account_info().key();
    fanout.name = args.name;
    fanout.total_shares = args.total_shares;
    fanout.total_available_shares = args.total_shares;
    fanout.total_inflow = 0;
    fanout.last_snapshot_amount = fanout.total_inflow;
    fanout.bump_seed = args.bump_seed;
    fanout.membership_model = model;
    fanout.membership_mint = if membership_mint.key()
        == Pubkey::try_from("So11111111111111111111111111111111111111112").unwrap()
    {
        None
    } else {
        Some(membership_mint.key())
    };
    match fanout.membership_model {
        MembershipModel::Wallet | MembershipModel::NFT => {
            fanout.membership_mint = None;
            fanout.total_staked_shares = None;
        }
        MembershipModel::Token => {
            fanout.total_shares = membership_mint.supply;
            fanout.total_available_shares = 0;
            if fanout.membership_mint.is_none() {
                return Err(HydraError::MintAccountRequired.into());
            }
            let mint = &ctx.accounts.membership_mint;
            fanout.total_staked_shares = Some(0);
            if !mint.is_initialized {
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let accounts = anchor_spl::token::InitializeMint {
                    mint: mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, accounts);
                anchor_spl::token::initialize_mint(
                    cpi_ctx,
                    0,
                    &ctx.accounts.authority.to_account_info().key(),
                    Some(&ctx.accounts.authority.to_account_info().key()),
                )?;
            }
        }
    };

    Ok(())
}
