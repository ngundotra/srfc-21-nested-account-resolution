//! This library provides a way for Solana programs to request additional accounts
//! for an instruction, according to sRFC 21.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_lang::solana_program::program::MAX_RETURN_DATA;
use anchor_lang::solana_program::{
    hash,
    program::{get_return_data, invoke, invoke_signed, set_return_data},
};

#[derive(Debug, Copy, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
#[repr(C)]
pub struct IAccountMeta {
    pub pubkey: Pubkey,
    pub writable: u8,
}

pub const MAX_ACCOUNTS: usize = 30;

#[zero_copy]
#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct AdditionalAccounts {
    pub protocol_version: u8,
    pub has_more: u8,
    pub _padding_1: [u8; 2],
    pub num_accounts: u32,
    pub accounts: [Pubkey; MAX_ACCOUNTS],
    pub writable_bits: [u8; MAX_ACCOUNTS],
    pub _padding_2: [u8; 26],
}

impl Default for AdditionalAccounts {
    fn default() -> Self {
        Self {
            protocol_version: 0,
            has_more: 0,
            _padding_1: [0u8; 2],
            num_accounts: 0u32,
            accounts: [Pubkey::default(); MAX_ACCOUNTS],
            writable_bits: [0u8; MAX_ACCOUNTS],
            _padding_2: [0u8; 26],
        }
    }
}

impl AdditionalAccounts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_space_available(&self) -> bool {
        MAX_ACCOUNTS - self.num_accounts as usize > 0
    }

    pub fn set_has_more(&mut self, has_more: bool) {
        self.has_more = match has_more {
            true => 1,
            false => 0,
        };
    }

    pub fn add_account(&mut self, pubkey: &Pubkey, writable: bool) -> Result<()> {
        if self.num_accounts >= MAX_ACCOUNTS as u32 {
            msg!("Cannot write another account");
            return Err(ProgramError::InvalidInstructionData.into());
        }

        self.accounts[self.num_accounts as usize] = *pubkey;
        self.writable_bits[self.num_accounts as usize] = match writable {
            true => 1,
            false => 0,
        };
        self.num_accounts += 1;
        Ok(())
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&Pubkey, bool)> {
        let num_accounts = self.num_accounts as usize;
        self.accounts[0..num_accounts]
            .iter()
            .zip(self.writable_bits[0..num_accounts].iter())
            .map(|(pubkey, writable)| {
                (
                    pubkey,
                    match writable {
                        0 => false,
                        1 => true,
                        _ => panic!("Invalid writable bit"),
                    },
                )
            })
    }

    pub fn iter_from(&self, start: usize) -> impl DoubleEndedIterator<Item = (&Pubkey, bool)> {
        let num_accounts = self.num_accounts as usize;
        self.accounts[start..num_accounts]
            .iter()
            .zip(self.writable_bits[0..num_accounts].iter())
            .map(|(pubkey, writable)| {
                (
                    pubkey,
                    match writable {
                        0 => false,
                        1 => true,
                        _ => panic!("Invalid writable bit"),
                    },
                )
            })
    }

    pub fn from_return_data(data: &[u8]) -> Result<&Self> {
        if data.len() != MAX_RETURN_DATA {
            msg!("Invalid return data length");
            return Err(ProgramError::InvalidAccountData.into());
        }
        Ok(bytemuck::from_bytes::<AdditionalAccounts>(&data))
    }
}

/// Resolves the page of accounts for a particular instruction
#[inline(never)]
pub fn resolve_additional_accounts<'info, C1: ToAccountInfos<'info> + ToAccountMetas>(
    ix_name: String,
    ctx: &CpiContext<'_, '_, '_, 'info, C1>,
    args: &[u8],
    log_info: bool,
) -> Result<AdditionalAccounts> {
    call_preflight_interface_function(ix_name.clone(), &ctx, &args)?;

    let program_key = ctx.program.key();
    let (key, program_data) = get_return_data().unwrap();
    assert_eq!(key, program_key);

    let program_data = program_data.as_slice();
    if log_info {
        msg!("Return data length: {}", program_data.len());
    }

    // Program return data actually may be unaligned on the stack
    // so we can't do our normal bytemuck::from_bytes call here
    let accs: AdditionalAccounts = bytemuck::pod_read_unaligned::<AdditionalAccounts>(program_data);
    if log_info {
        msg!(
            "Accounts has more: {} {}",
            accs.has_more,
            accs.accounts.len()
        );
    }
    Ok(accs)
}

/// Returns the additional accounts needed to execute the instruction
/// Will only return up to MAX_ACCOUNTS accounts.
pub fn identify_additional_accounts<'info, C1: ToAccountInfos<'info> + ToAccountMetas>(
    ix_name: String,
    ctx: &CpiContext<'_, '_, '_, 'info, C1>,
    args: &[u8],
    log_info: bool,
) -> Result<Vec<AdditionalAccounts>> {
    if log_info {
        msg!("Preflight {}", &ix_name);
    }

    let mut additional_accounts: Vec<AdditionalAccounts> = vec![];

    // This is really meant to page all accounts, page by page
    // to get all the account metas to send
    let mut has_more = true;
    while has_more {
        let accs = resolve_additional_accounts(ix_name.clone(), ctx, args, log_info)?;

        additional_accounts.push(accs);

        // If we are missing any of the requested accounts, we should exit
        let mut should_exit = false;
        accs.iter().rev().for_each(|(acc, _writable)| {
            let mut found = false;
            ctx.remaining_accounts.iter().rev().for_each(|account| {
                if account.key == acc {
                    found = true;
                }
            });
            if !found {
                should_exit = true;
            }
        });
        if should_exit {
            msg!("Missing account(s)");
            break;
        }

        has_more = accs.has_more == 1;
    }

    Ok(additional_accounts)
}

/// This calls the preflight function on the target program (defined on the ctx)
pub fn call_preflight_interface_function<'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    function_name: String,
    ctx: &CpiContext<'_, '_, '_, 'info, T>,
    args: &[u8],
) -> Result<()> {
    // setup
    sol_log_compute_units();
    let mut ix_data: Vec<u8> =
        hash::hash(format!("global:preflight_{}", &function_name).as_bytes()).to_bytes()[..8]
            .to_vec();

    ix_data.extend_from_slice(args);

    let mut ix_account_metas = ctx.accounts.to_account_metas(Some(false));
    ix_account_metas.extend(ctx.remaining_accounts.to_account_metas(None));

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts: ix_account_metas,
        data: ix_data,
    };
    sol_log_compute_units();
    msg!("Preflighted...");

    // execute
    let mut ix_ais = ctx.accounts.to_account_infos();
    ix_ais.extend(ctx.remaining_accounts.to_account_infos());
    invoke(&ix, &ix_ais)?;
    Ok(())
}

pub fn call_interface_function_raw(
    program_key: &Pubkey,
    function_name: String,
    args: &[u8],
    metas: Vec<AccountMeta>,
    accounts: &[AccountInfo],
    signer_seeds: &[&[&[u8]]],
    log_info: bool,
) -> Result<()> {
    let mut ix_data: Vec<u8> =
        hash::hash(format!("global:{}", &function_name).as_bytes()).to_bytes()[..8].to_vec();
    ix_data.extend_from_slice(&args);

    if log_info {
        msg!("Account Metas creation...");
        sol_log_compute_units();
    }
    if log_info {
        sol_log_compute_units();
        msg!("Account Metas created...");
    }

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: *program_key,
        accounts: metas,
        data: ix_data,
    };

    // Oddly enough, we only need to specify the account metas
    // we can just throw the account infos in there and account metas
    // will specify ordering & filtering
    if log_info {
        msg!("Finished creating context...");
        sol_log_compute_units();
    }

    invoke_signed(&ix, &accounts, &signer_seeds)?;
    Ok(())
}

/// This calls the main function on the target program, and passes along the requested
/// account_metas from the preflight function
pub fn call_interface_function<'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    function_name: String,
    ctx: CpiContext<'_, '_, '_, 'info, T>,
    args: &[u8],
    additional_accounts: &mut dyn Iterator<Item = (&Pubkey, bool)>,
    log_info: bool,
) -> Result<()> {
    if log_info {
        msg!("Creating interface context...");
        sol_log_compute_units();
    }
    // setup
    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let mut ix_data: Vec<u8> =
        hash::hash(format!("global:{}", &function_name).as_bytes()).to_bytes()[..8].to_vec();
    ix_data.extend_from_slice(&args);

    if log_info {
        msg!("Account Metas creation...");
        sol_log_compute_units();
    }
    let mut ix_account_metas = ctx.accounts.to_account_metas(None);
    ix_account_metas.append(
        additional_accounts
            .map(|(acc, writable)| {
                if writable {
                    AccountMeta::new(*acc, false)
                } else {
                    AccountMeta::new_readonly(*acc, false)
                }
            })
            .collect::<Vec<AccountMeta>>()
            .as_mut(),
    );
    if log_info {
        sol_log_compute_units();
        msg!("Account Metas created...");
    }

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts: ix_account_metas,
        data: ix_data,
    };

    let mut ix_ais: Vec<AccountInfo> = ctx.accounts.to_account_infos();
    if log_info {
        msg!("IX accounts: {:?}", &ix_ais.len());
        msg!("Account Info creation...");
        sol_log_compute_units();
    }
    // Oddly enough, we only need to specify the account metas
    // we can just throw the account infos in there and account metas
    // will specify ordering & filtering (?)
    ix_ais.extend_from_slice(&remaining_accounts);
    if log_info {
        sol_log_compute_units();
        msg!("Account Infos created...");
    }

    if log_info {
        msg!("IX accounts: {:?}", &ix_ais.len());
        // ix_ais.iter().into_iter().for_each(|ai| {
        //     msg!(
        //         "Account: {:?}, {:?}, {:?}, {:?}",
        //         ai.key,
        //         ai.owner,
        //         ai.is_signer,
        //         ai.is_writable
        //     )
        // });
        // msg!("Signer seeds: {:?}", &ctx.signer_seeds);
    }

    if log_info {
        msg!("Finished creating context...");
        sol_log_compute_units();
    }

    // execute
    invoke_signed(&ix, &ix_ais, &ctx.signer_seeds)?;
    Ok(())
}

pub fn get_delimiter(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&["DELIMITER".as_ref()], program_id).0
}

/// Calls an instruction on a program that complies with the additional accounts interface
///
/// Expects ctx.remaining accounts to have all possible accounts in order to resolve
/// the accounts requested from the preflight function
#[inline(never)]
pub fn call<'info, C1: ToAccountInfos<'info> + ToAccountMetas>(
    ix_name: String,
    ctx: CpiContext<'_, '_, '_, 'info, C1>,
    args: Vec<u8>,
    delimiter: Pubkey,
    num_accounts_consumed: u8,
    log_info: bool,
) -> Result<u8> {
    // preflight
    let mut accounts = ctx.accounts.to_account_infos();
    let mut metas = ctx.accounts.to_account_metas(None);

    if log_info {
        msg!("Identifying additional accounts...");
        sol_log_compute_units();
    }
    let mut used_accounts = 0;
    for acc in ctx.remaining_accounts[num_accounts_consumed as usize..].iter() {
        used_accounts += 1;
        if *acc.key != delimiter {
            accounts.push(acc.clone());
            metas.push(AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            });
        } else {
            if log_info {
                msg!("Found delimiter");
            }
            break;
        }
    }

    // execute
    if log_info {
        sol_log_compute_units();
        msg!("Execute {}", &ix_name);
    }
    call_interface_function_raw(
        ctx.program.key,
        ix_name.clone(),
        &args,
        metas,
        &accounts,
        ctx.signer_seeds,
        log_info,
    )?;
    Ok(num_accounts_consumed + used_accounts)
}

pub fn forward_return_data(expected_program_key: &Pubkey) {
    let (key, return_data) = get_return_data().unwrap();
    assert_eq!(key, *expected_program_key);
    set_return_data(&return_data);
}

pub trait InterfaceInstruction {
    fn instruction_name() -> String;
}
