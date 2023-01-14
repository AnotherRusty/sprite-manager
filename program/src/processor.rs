use crate::{
    error::SpriteManagerError,
    instruction::{SpriteManagerInstruction, StoreSpriteArgs},
    state::{Key, SolanaAccount, Sprite, SpriteAccount, PREFIX},
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::state::{EscrowAuthority, ESCROW_POSTFIX};
use mpl_utils::{
    assert_derivation, assert_owned_by, assert_signer, create_or_allocate_account_raw,
    resize_or_reallocate_account_raw,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_memory::sol_memcpy,
    program_pack::Pack,
    pubkey::Pubkey,
};

pub struct Processor;
impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction: SpriteManagerInstruction =
            SpriteManagerInstruction::try_from_slice(instruction_data)?;
        match instruction {
            SpriteManagerInstruction::CreateSpriteAccount => {
                process_create_sprite_account(program_id, accounts)
            }
            SpriteManagerInstruction::StoreSprite(args) => {
                process_store_sprite(program_id, accounts, args)
            }
        }
    }
}

pub fn process_create_sprite_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let sprite_pda_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let _tm_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let sysvar_ix_account_info = next_account_info(account_info_iter)?;

    let sprite_pda_bump = assert_derivation(
        program_id,
        sprite_pda_info,
        &[PREFIX.as_bytes(), mint_info.key.as_ref()],
        SpriteManagerError::DerivedKeyInvalid,
    )?;

    assert_signer(creator_info)?;
    assert_owned_by(
        escrow_info,
        system_program_info.key,
        SpriteManagerError::AlreadyInitialized,
    )?;
    if !escrow_info.data_is_empty() {
        return Err(SpriteManagerError::AlreadyInitialized.into());
    }

    let sprite_signer_seeds = &[
        PREFIX.as_bytes(),
        mint_info.key.as_ref(),
        &[sprite_pda_bump],
    ];

    let sprite_account = SpriteAccount {
        key: Key::SpriteAccount,
        base_mint: *mint_info.key,
        ..SpriteAccount::default()
    };

    let serialized_data = sprite_account
        .try_to_vec()
        .map_err(|_| SpriteManagerError::FailedToSerialize)?;

    create_or_allocate_account_raw(
        *program_id,
        sprite_pda_info,
        system_program_info,
        creator_info,
        serialized_data.len(),
        sprite_signer_seeds,
    )?;

    sol_memcpy(
        &mut **sprite_pda_info
            .try_borrow_mut_data()
            .map_err(|_| SpriteManagerError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    let create_escrow_account_ix = mpl_token_metadata::escrow::create_escrow_account(
        mpl_token_metadata::ID,
        *escrow_info.key,
        *metadata_info.key,
        *mint_info.key,
        *token_account_info.key,
        *edition_info.key,
        *creator_info.key,
        Some(*sprite_pda_info.key),
    );

    let account_infos = vec![
        escrow_info.clone(),
        metadata_info.clone(),
        mint_info.clone(),
        token_account_info.clone(),
        edition_info.clone(),
        creator_info.clone(),
        system_program_info.clone(),
        sprite_pda_info.clone(),
        sysvar_ix_account_info.clone(),
    ];

    msg!("Creating token escrow.");
    invoke_signed(
        &create_escrow_account_ix,
        &account_infos,
        &[sprite_signer_seeds],
    )?;

    Ok(())
}

pub fn process_store_sprite(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: StoreSpriteArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let base_mint_info = next_account_info(account_info_iter)?;
    let sprite_mint_info = next_account_info(account_info_iter)?;
    let sprite_mint_src_info = next_account_info(account_info_iter)?;
    let sprite_mint_dst_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let sprite_pda_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let _associated_token_account_program_info = next_account_info(account_info_iter)?;

    assert_signer(payer_info)?;

    let mut escrow_seeds = vec![
        mpl_token_metadata::state::PREFIX.as_bytes(),
        mpl_token_metadata::ID.as_ref(),
        base_mint_info.key.as_ref(),
    ];

    let escrow_auth = EscrowAuthority::Creator(*sprite_pda_info.key);
    for seed in escrow_auth.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_POSTFIX.as_bytes());

    assert_derivation(
        &mpl_token_metadata::ID,
        escrow_info,
        &escrow_seeds,
        SpriteManagerError::DerivedKeyInvalid,
    )?;

    // Deserialize the token accounts and perform checks.
    let attribute_src = spl_token::state::Account::unpack(&sprite_mint_src_info.data.borrow())?;
    assert!(attribute_src.mint == *sprite_mint_info.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= 1);

    let sprite_seeds = &[PREFIX.as_bytes(), base_mint_info.key.as_ref()];

    let sprite_bump_seed = assert_derivation(
        program_id,
        sprite_pda_info,
        sprite_seeds,
        SpriteManagerError::DerivedKeyInvalid,
    )?;

    let _sprite_signer_seeds = &[
        PREFIX.as_bytes(),
        base_mint_info.key.as_ref(),
        &[sprite_bump_seed],
    ];

    // Only try to create the ATA if the account doesn't already exist.
    if *sprite_mint_dst_info.owner != spl_token::ID && sprite_mint_dst_info.lamports() == 0 {
        // Allocate the escrow accounts new ATA.
        let create_escrow_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                escrow_info.key,
                sprite_mint_info.key,
            );

        invoke(
            &create_escrow_ata_ix,
            &[
                payer_info.clone(),
                sprite_mint_dst_info.clone(),
                escrow_info.clone(),
                sprite_mint_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
            ],
        )?;

        // Transfer the token from the current owner into the escrow.
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            sprite_mint_src_info.key,
            sprite_mint_dst_info.key,
            payer_info.key,
            &[payer_info.key],
            1,
        )?;

        invoke(
            &transfer_ix,
            &[
                sprite_mint_src_info.clone(),
                sprite_mint_dst_info.clone(),
                payer_info.clone(),
                token_program_info.clone(),
            ],
        )?;
    }

    let mut sprite_account = SpriteAccount::from_account_info(sprite_pda_info)?;
    sprite_account.sprites.push(Sprite {
        name: args.name,
        description: args.description,
        perspective_tags: args.perspective_tags,
        style_tags: args.style_tags,
        custom_tags: args.custom_tags,
        mint: *sprite_mint_info.key,
    });

    let serialized_data = sprite_account
        .try_to_vec()
        .map_err(|_| SpriteManagerError::FailedToSerialize)?;

    resize_or_reallocate_account_raw(
        sprite_pda_info,
        payer_info,
        system_program_info,
        serialized_data.len(),
    )?;

    sol_memcpy(
        &mut **sprite_pda_info
            .try_borrow_mut_data()
            .map_err(|_| SpriteManagerError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}
