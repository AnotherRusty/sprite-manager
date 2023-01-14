use solana_program::pubkey::Pubkey;

use crate::state::PREFIX;

/// Sprite account PDA seeds
///     "sprite",
///     mint.key.as_ref(),
pub fn find_sprite_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PREFIX.as_bytes(), mint.as_ref()], &crate::id())
}
