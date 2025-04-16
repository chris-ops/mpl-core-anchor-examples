use anchor_lang::prelude::*;
use anchor_spl::{token::Token, token_interface::Mint as IMint};
use anchor_spl::associated_token::{self, AssociatedToken, Create};
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
pub const SEED: &str = "AUTH";
use mpl_core::{
    instructions::{AddPluginV1CpiBuilder, TransferV1CpiBuilder, CreateV1CpiBuilder},
    ID as MPL_CORE_ID,
};
use anchor_lang::prelude::Pubkey;
declare_id!("6WzfBWA9hkAdQW24Tt8UQ9h8m5zQyMVWAf8xwn7ic6Vb");

#[program]
pub mod nft_manager {
    use mpl_core::types::PluginAuthority;
    use spl_transfer_hook_interface::instruction::ExecuteInstruction;
    use anchor_lang::{solana_program::system_instruction::{self, SystemInstruction}, system_program::{self, create_account, CreateAccount, Transfer}};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let manager = &mut ctx.accounts.manager;
        manager.owner = ctx.accounts.owner.key();
        Ok(())
    }

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
    let account_metas = vec![
        ExtraAccountMeta::new_with_pubkey(&Pubkey::new_from_array(mpl_core::ID.to_bytes()), false, true)?,
        ExtraAccountMeta::new_with_pubkey(&associated_token::ID, false, true)?,
        ExtraAccountMeta::new_with_pubkey(&system_program::ID, false, false)?,
    ]; 

        let account_size: u64 = 3;
        let lamports = Rent::get()?.minimum_balance(3 as usize);
        let mint = ctx.accounts.mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"extra-account-metas",
            &mint.as_ref(),
            &[ctx.bumps.extra_account_meta_list],
        ]];

        create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.extra_account_meta_list.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            lamports,
            account_size,
            ctx.program_id,
        )?;

        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &account_metas,
        )?;
        Ok(())
    }

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {

        // first we must check the direction of the transfer.
        // we can do that by comparing the source/destination account to the ATA resulting by the mint and the owner.
        // if the source is the owner, then we are transferring out = transfer nft
        // if the destination is the owner, then we are transferring in = mint nft

        let source = ctx.accounts.source.key();
        let destination = ctx.accounts.destination.key();
        let owner = ctx.accounts.owner.key();
        let mint = ctx.accounts.mint.key();

        // derive the pda address of the owner and the mint
        let (pda, bump) = Pubkey::find_program_address(
            &[
                owner.as_ref(),
                mint.as_ref(),
            ],
            &ctx.accounts.associated_token_program.key(),
        );
        let amount = amount / 9;
        // repeat the mint/transfer logic for amount time
        for _ in 0..amount {
            // if the source is the owner, then we are transferring out = transfer nft
            if source == owner {
                msg!("Transferring NFT");
                let manager = &mut ctx.accounts.manager;

                // divide the amount by 9 to get the integer value
                // check if the amount is greater than 1
        
                let assets = [
                    (&ctx.accounts.asset_0, 0),
                    (&ctx.accounts.asset_1, 1),
                    (&ctx.accounts.asset_2, 2),
                    (&ctx.accounts.asset_3, 3),
                    (&ctx.accounts.asset_4, 4),
                ];
        
                let mut asset_to_transfer = None;
                let mut index_to_transfer: Option<u64> = None;
        
                for (asset, index) in assets.iter() {
                    let byte_idx = (*index / 8) as usize;
                    let bit_idx = (*index % 8) as u8;
                        // Check if account is unused (empty or not an Asset)
                        if asset.to_account_info().data.borrow().len() > 0 {
                            asset_to_transfer = Some(asset);
                            index_to_transfer = Some(*index);
                            break;
                    }
                }
        
                // if-else to find the corresponding bump seed
                let bump = match index_to_transfer {
                    Some(index) => {
                        match index {
                            0 => ctx.bumps.asset_0,
                            1 => ctx.bumps.asset_1,
                            2 => ctx.bumps.asset_2,
                            3 => ctx.bumps.asset_3,
                            4 => ctx.bumps.asset_4,
                            5 => ctx.bumps.asset_5,
                            6 => ctx.bumps.asset_6,
                            7 => ctx.bumps.asset_7,
                            8 => ctx.bumps.asset_8,
                            9 => ctx.bumps.asset_9,
                            _ => return Err(error!(ErrorCode::NoAvailableIndex)),
                        }
                    }
                    None => return Err(error!(ErrorCode::NoAvailableIndex)),
                };
                let asset = match asset_to_transfer {
                    Some(asset) => asset,
                    None => return Err(error!(ErrorCode::NoNFTsAvailable)),
                };
        
                let binding = ctx.accounts.owner.key();
                let manager_seeds = &[b"manager", binding.as_ref(), &[ctx.bumps.manager]];
                // index must be a value and not an option, otherwise throw.
                let index = match index_to_transfer {
                    Some(index) => index,
                    None => return Err(error!(ErrorCode::NoAvailableIndex)),
                };
                let bindingid = index.to_le_bytes();
        
                let asset_seeds = &[b"nft", binding.as_ref(), &bindingid, &[bump]];
                let signer_seeds: &[&[&[u8]]] = &[manager_seeds, asset_seeds];
                msg!("Transferring NFT...");
                msg!("Asset: {}", asset.key());
                //index
                msg!("Index: {}", index);
                TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program)
                    .asset(&asset.to_account_info())
                    .payer(&ctx.accounts.owner.to_account_info())
                    .new_owner(&manager.to_account_info())
                    .invoke_signed(signer_seeds)?;
            }
            // if the destination is the owner, then we are transferring in = mint nft
            else if destination == owner {
                msg!("Minting NFT");
                let manager = &mut ctx.accounts.manager;
                let assets = [
                    (&ctx.accounts.asset_0, 0),
                    (&ctx.accounts.asset_1, 1),
                    (&ctx.accounts.asset_2, 2),
                    (&ctx.accounts.asset_3, 3),
                    (&ctx.accounts.asset_4, 4),
                    // Extend to asset_99...
                ];
        
                let mut asset_to_mint = None;
                let mut index_to_mint: Option<u64> = None;
        
                let mut owned_by_manager: u64 = 0;
        
                for (asset, index) in assets.iter() {
                    let byte_idx = (*index / 8) as usize;
                    let bit_idx = (*index % 8) as u8;
                        // Check if account is unused (empty or not an Asset)
                        if asset.to_account_info().data.borrow().is_empty() {
                            asset_to_mint = Some(asset);
                            index_to_mint = Some(*index);
                            break;
        
                        // if owned by manager, increment the owned_by_manager
                        }
                         if asset.to_account_info().key == &manager.owner {
                            owned_by_manager += 1;
                        }
                }
        
                // if no account is empty, then all accounts are used.
                // we must now check how many are owned by the manager.
                if asset_to_mint.is_none() {
                    // Check if the manager has any NFTs
                    if owned_by_manager >= 10 {
                        return Err(error!(ErrorCode::NoAvailableIndex));
                    }
                }
        
                // if-else to find the corresponding bump seed
                let bump = match index_to_mint {
                    Some(index) => {
                        match index {
                            0 => ctx.bumps.asset_0,
                            1 => ctx.bumps.asset_1,
                            2 => ctx.bumps.asset_2,
                            3 => ctx.bumps.asset_3,
                            4 => ctx.bumps.asset_4,
                            5 => ctx.bumps.asset_5,
                            6 => ctx.bumps.asset_6,
                            7 => ctx.bumps.asset_7,
                            8 => ctx.bumps.asset_8,
                            9 => ctx.bumps.asset_9,
                            _ => return Err(error!(ErrorCode::NoAvailableIndex)),
                        }
                    }
                    None => return Err(error!(ErrorCode::NoAvailableIndex)),
                };
        
                let asset = match asset_to_mint {
                    Some(asset) => asset,
                    None => return Err(error!(ErrorCode::NoAvailableIndex)),
                };
        
                // index must be a value and not an option, otherwise throw.
                let index = match index_to_mint {
                    Some(index) => index,
                    None => return Err(error!(ErrorCode::NoAvailableIndex)),
                };
        
                // Mint the NFT with MPL Core
                let binding = ctx.accounts.owner.key();
                let manager_seeds = &[b"manager", binding.as_ref(), &[ctx.bumps.manager]];
                let bindingid = index.to_le_bytes();
                let asset_seeds = &[b"nft", binding.as_ref(), &bindingid, &[bump]];
                let signer_seeds: &[&[&[u8]]] = &[manager_seeds, asset_seeds];
                // log the address of asset
                msg!(&asset.key().to_string());
                CreateV1CpiBuilder::new(&ctx.accounts.mpl_core_program)
                    .asset(&asset.to_account_info())
                    .authority(Some(&manager.to_account_info()))
                    .owner(Some(&ctx.accounts.owner.to_account_info()))
                    .payer(&ctx.accounts.owner)
                    .system_program(&ctx.accounts.system_program)
                    .name("name".to_string())
                    .uri("uri".to_string())
                    .plugins(vec![
                        mpl_core::types::PluginAuthorityPair {
                            plugin: mpl_core::types::Plugin::PermanentFreezeDelegate(
                                mpl_core::types::PermanentFreezeDelegate {
                                    frozen: true,
                                },
                            ),
                            authority: Some(PluginAuthority::Address { address: manager.owner }),
                        },
                        mpl_core::types::PluginAuthorityPair {
                            plugin: mpl_core::types::Plugin::PermanentTransferDelegate(
                                mpl_core::types::PermanentTransferDelegate {},
                            ),
                            authority: Some(PluginAuthority::Address { address: manager.owner }),
                        },
                    ])
                    .invoke_signed(signer_seeds)?;
            }
        }
        Ok(())
    }

}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 32 * 100,
        seeds = [b"manager", owner.key().as_ref()],
        bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint: InterfaceAccount<'info, IMint>,
    /// CHECK:
    #[account(mut, seeds = [b"extra-account-metas", mint.key().as_ref()], bump)]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut, executable, address = MPL_CORE_ID)]
    /// CHECK: asd
    pub core_program: UncheckedAccount<'info>,
    /// CHECK: asd
    pub log_wrapper: UncheckedAccount<'info>,
    #[account(seeds = [SEED.as_bytes()], bump)]
    /// CHECK: asd
    pub compression_program: UncheckedAccount<'info>,
}


#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferHook<'info> {
    /// CHECK: asd
    pub source: AccountInfo<'info>,
    /// CHECK: asd
    pub mint: AccountInfo<'info>,
    /// CHECK: asd
    pub destination: AccountInfo<'info>,
    /// CHECK: asd
    pub owner: UncheckedAccount<'info>,
    #[account(seeds = [b"extra-account-metas", mint.key().as_ref()], bump)]
    /// CHECK: asd
    pub extra_account: AccountInfo<'info>,
    /// CHECK: asd
    pub mpl_core_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
        /// CHECK: asd
        #[account(
            mut,
            seeds = [b"manager", owner.key().as_ref()],
            bump,
            has_one = owner
        )]
        pub manager: Account<'info, Manager>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 0u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_0: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 1u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_1: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 2u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_2: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 3u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_3: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 4u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_4: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 5u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_5: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 6u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_6: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 7u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_7: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 8u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_8: AccountInfo<'info>,
        #[account(
            mut,
            seeds = [b"nft", owner.key().as_ref(), 9u64.to_le_bytes().as_ref()],
            bump
        )]
        /// CHECK:: aaaaaaaaaaaaaa
        pub asset_9: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("No NFTs available to transfer")]
    NoNFTsAvailable,
    #[msg("Invalid NFT")]
    InvalidNFT,
    #[msg("No available index to mint")]
    NoAvailableIndex,
}

#[account]
pub struct Manager {
    pub owner: Pubkey,
}
