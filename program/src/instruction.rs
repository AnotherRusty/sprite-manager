use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use crate::state::{PerspectiveTags, StyleTags};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct StoreSpriteArgs {
    pub name: String,
    pub description: String,
    pub perspective_tags: Vec<PerspectiveTags>,
    pub style_tags: Vec<StyleTags>,
    pub custom_tags: Vec<String>,
}

#[derive(Debug, Clone, ShankInstruction, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
pub enum SpriteManagerInstruction {
    /// Instruction for creating a sprite storage account
    #[account(0, writable, name = "escrow", desc = "Escrow account")]
    #[account(1, writable, name = "metadata", desc = "Metadata account")]
    #[account(2, name = "mint", desc = "Mint account")]
    #[account(3, writable, name = "token_account", desc = "Token account (base token)")]
    #[account(4, name = "edition", desc = "Edition account")]
    #[account(5, writable, name = "sprite_pda", desc = "Sprite PDA account")]
    #[account(6, signer, name = "creator", desc = "The creator of the global sprite account")]
    #[account(7, name = "token_metadata_program", desc = "Token Metadata program")]
    #[account(8, name = "system_program", desc = "System program")]
    #[account(9, name="sysvar_instructions", desc="Instructions sysvar account")]
    CreateSpriteAccount,

    /// Instruction for storing a sprite
    #[account(0, writable, name = "escrow", desc = "Escrow account")]
    #[account(1, name = "base_mint", desc = "Mint account of the base token")]
    #[account(2, name = "sprite_mint", desc = "Mint account of the sprite token")]
    #[account(3, name = "sprite_mint_src", desc = "Source account of the sprite token")]
    #[account(4, name = "sprite_mint_dst", desc = "Destination account of the sprite token")]
    #[account(5, writable, signer, name="payer", desc="The creator of the account and manager of the sprite")]
    #[account(6, writable, name="sprite_pda", desc = "The PDA for sprite data")]
    #[account(7, name = "system_program", desc = "System program")]
    #[account(8, name = "spl_token", desc = "Token program")]
    #[account(9, name = "spl_associated_token", desc = "Associated token account program")]
    StoreSprite(StoreSpriteArgs),
}

#[allow(clippy::too_many_arguments)]
pub fn create_sprite_account(
    program_id: &Pubkey,
    escrow: &Pubkey,
    metadata: &Pubkey,
    mint: &Pubkey,
    token_account: &Pubkey,
    edition: &Pubkey,
    sprite_account: &Pubkey,
    creator: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*escrow, false),
        AccountMeta::new(*metadata, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new_readonly(*edition, false),
        AccountMeta::new(*sprite_account, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new_readonly(mpl_token_metadata::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: SpriteManagerInstruction::CreateSpriteAccount
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn store_sprite(
    program_id: &Pubkey,
    escrow: &Pubkey,
    base_mint: &Pubkey,
    sprite_mint: &Pubkey,
    sprite_mint_src: &Pubkey,
    sprite_mint_dst: &Pubkey,
    payer: &Pubkey,
    sprite_account: &Pubkey,
    name: String,
    description: String,
    perspective_tags: Vec<PerspectiveTags>,
    style_tags: Vec<StyleTags>,
    custom_tags: Vec<String>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*escrow, false),
        AccountMeta::new_readonly(*base_mint, false),
        AccountMeta::new_readonly(*sprite_mint, false),
        AccountMeta::new(*sprite_mint_src, false),
        AccountMeta::new(*sprite_mint_dst, false),
        AccountMeta::new_readonly(*payer, true),
        AccountMeta::new(*sprite_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: SpriteManagerInstruction::StoreSprite(StoreSpriteArgs {
            name,
            description,
            perspective_tags,
            style_tags,
            custom_tags,
        })
        .try_to_vec()
        .unwrap(),
    }
}
