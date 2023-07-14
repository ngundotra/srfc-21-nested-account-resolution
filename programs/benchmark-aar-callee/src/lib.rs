use additional_accounts_request::{IAccountMeta, PreflightPayload};
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::program::set_return_data;
use anchor_lang::Discriminator;

declare_id!("8hKjTVHaCE4U2zMYVx5eu5P9MTCU2imhvZZU31jDnYNA");

fn get_pubkey(num: u32) -> Pubkey {
    Pubkey::find_program_address(&["key".as_ref(), &num.to_le_bytes()], &crate::id()).0
}

fn create_accounts(num_accounts: u32) -> Vec<IAccountMeta> {
    let mut accounts: Vec<IAccountMeta> = vec![];
    for idx in 0..num_accounts {
        let pubkey = get_pubkey(idx);
        accounts.push(IAccountMeta {
            pubkey,
            signer: false,
            writable: false,
        });
    }
    accounts
}

#[program]
pub mod benchmark_aar_callee {
    use std::thread::current;

    use anchor_lang::solana_program::borsh::try_from_slice_unchecked;
    use anchor_lang::solana_program::program::invoke;
    use anchor_lang::solana_program::sysvar::{rent::Rent, Sysvar};
    use anchor_lang::solana_program::{log::sol_log_compute_units, system_instruction};

    use super::*;

    pub fn preflight_transfer(
        ctx: Context<TransferNoExtraAccounts>,
        num_accounts: u32,
    ) -> Result<()> {
        // Setup the base accounts
        let event_authority =
            Pubkey::find_program_address(&[b"__event_authority"], &ctx.program_id).0;
        let mut accounts = vec![
            IAccountMeta {
                pubkey: event_authority,
                signer: false,
                writable: false,
            },
            IAccountMeta {
                pubkey: *ctx.program_id,
                signer: false,
                writable: false,
            },
        ];

        let additional_accounts = create_accounts(num_accounts);
        accounts.extend_from_slice(&additional_accounts);

        set_return_data(
            &PreflightPayload {
                accounts,
                has_more: false,
            }
            .try_to_vec()?,
        );
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, num_accounts: u32) -> Result<()> {
        for idx in 0..num_accounts {
            let acct = ctx.remaining_accounts.get(idx as usize).unwrap();
            if acct.key() != get_pubkey(idx) {
                msg!("Invalid account {}", idx);
                return Err(ProgramError::InvalidInstructionData.into());
            }
        }
        Ok(())
    }

    /// Transfers all nodes in a linked list to a different owner
    pub fn transfer_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferNested<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        let mut current_node = &mut ctx.accounts.head_node;
        msg!("current: {:?}", &current_node.owner);
        current_node.owner = destination;
        current_node.exit(&crate::id())?;

        let mut current_node = current_node.clone().into_inner();

        let mut accounts_iter = ctx.remaining_accounts.into_iter();
        while current_node.next.is_some() {
            let next_node = current_node.next.unwrap();
            let next_acct = next_account_info(&mut accounts_iter)?;

            let mut next_node_acct = Account::<Node>::try_from(next_acct)?;
            next_node_acct.owner = destination;
            next_node_acct.exit(&crate::id())?;

            current_node = next_node_acct.clone().into_inner();
        }

        Ok(())
    }

    pub fn preflight_transfer_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferNested<'info>>,
        destination: Pubkey,
    ) -> Result<()> {
        let mut accounts: Vec<IAccountMeta> = vec![];
        let mut has_more: bool = false;

        let mut accounts_iter = ctx.remaining_accounts.into_iter();

        let mut current_node = ctx.accounts.head_node.to_owned();
        while current_node.next.is_some() {
            let next_node = current_node.next.unwrap();
            match next_account_info(&mut accounts_iter) {
                Ok(acct) => {
                    if acct.key() != next_node {
                        msg!("Invalid account");
                        return Err(ProgramError::InvalidInstructionData.into());
                    } else {
                        current_node = Account::<Node>::try_from_unchecked(&acct)?;
                    }
                }
                _ => {
                    accounts.push(IAccountMeta {
                        pubkey: next_node,
                        signer: false,
                        writable: true,
                    });
                    has_more = true;
                    break;
                }
            }
        }

        set_return_data(&PreflightPayload { accounts, has_more }.try_to_vec()?);
        Ok(())
    }

    pub fn create_linked_list<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateLinkedList<'info>>,
        num: u32,
    ) -> Result<()> {
        let mut accounts_iter = ctx.remaining_accounts.into_iter();
        let mut prev_node: Option<Node> = None;
        let mut prev_ai: Option<&AccountInfo> = None;

        let payer = ctx.accounts.payer.to_account_info();
        for i in 0..num {
            let acct = next_account_info(&mut accounts_iter)?;

            let space: u64 = 8 + std::mem::size_of::<Node>() as u64;
            let lamports = Rent::get()?.minimum_balance(space as usize);
            let ix = system_instruction::create_account(
                ctx.accounts.payer.key,
                acct.key,
                lamports,
                space,
                &crate::id(),
            );
            invoke(&ix, &[payer.clone(), acct.clone()])?;

            let node = Node {
                id: i,
                next: None,
                owner: payer.key(),
            };

            if let Some(mut prev_node) = prev_node {
                prev_node.next = Some(acct.key());
                let mut data = Node::discriminator().to_vec();
                data.extend_from_slice(&prev_node.try_to_vec()?);

                let mut account_data = prev_ai.unwrap().try_borrow_mut_data()?;
                account_data[0..data.len()].copy_from_slice(&data);
            }
            prev_node = Some(node);
            prev_ai = Some(acct);
        }

        if let Some(mut prev_node) = prev_node {
            prev_node.next = None;
            let mut data = Node::discriminator().to_vec();
            data.extend_from_slice(&prev_node.try_to_vec()?);

            let mut account_data = prev_ai.unwrap().try_borrow_mut_data()?;
            account_data[0..data.len()].copy_from_slice(&data);
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[event_cpi]
pub struct Transfer {}

#[derive(Accounts)]
pub struct TransferNoExtraAccounts {}

#[derive(Accounts)]
pub struct TransferNested<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub head_node: Account<'info, Node>,
}

// Boilerplate to support calling `transfer_nested`
// This writes accounts to a `book` account, which contains pointer to next node
#[derive(Accounts)]
pub struct CreateLinkedList<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

// -- end --

#[derive(Debug)]
#[account]
pub struct Node {
    pub id: u32,
    pub owner: Pubkey,
    pub next: Option<Pubkey>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ExternalIAccountMeta {
    pubkey: Pubkey,
    signer: bool,
    writable: bool,
}
