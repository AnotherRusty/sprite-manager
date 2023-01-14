use borsh::{maybestd::io::Error as BorshError, BorshDeserialize, BorshSerialize};
use mpl_utils::assert_owned_by;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use std::io::ErrorKind;

use crate::error::SpriteManagerError;

pub const PREFIX: &str = "sprite";

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy, FromPrimitive)]
pub enum Key {
    Uninitialized,
    SpriteAccount,
}

impl Default for Key {
    fn default() -> Self {
        Key::Uninitialized
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum PerspectiveTags {
    RPG,
    TopDown,
    SideScroller,
    Platformer,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum StyleTags {
    Pixel,
    Vector,
    HandDrawn,
    Cartoon,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Sprite {
    pub name: String,
    pub description: String,
    pub perspective_tags: Vec<PerspectiveTags>,
    pub style_tags: Vec<StyleTags>,
    pub custom_tags: Vec<String>,
    pub mint: Pubkey,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, Default)]
pub struct SpriteAccount {
    pub key: Key,
    pub base_mint: Pubkey,
    pub sprites: Vec<Sprite>,
}

impl SolanaAccount for SpriteAccount {
    fn key() -> Key {
        Key::SpriteAccount
    }

    fn size() -> usize {
        0
    }
}

pub trait SolanaAccount: BorshDeserialize {
    fn key() -> Key;

    fn size() -> usize;

    fn is_correct_account_type(data: &[u8], data_type: Key) -> bool {
        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => key == data_type || key == Key::Uninitialized,
            None => false,
        }
    }

    fn pad_length(buf: &mut Vec<u8>) -> Result<(), SpriteManagerError> {
        let padding_length = Self::size()
            .checked_sub(buf.len())
            .ok_or(SpriteManagerError::NumericalOverflow)?;
        buf.extend(vec![0; padding_length]);
        Ok(())
    }

    fn safe_deserialize(mut data: &[u8]) -> Result<Self, BorshError> {
        if !Self::is_correct_account_type(data, Self::key()) {
            return Err(BorshError::new(ErrorKind::Other, "DataTypeMismatch"));
        }

        let result = Self::deserialize(&mut data)?;

        Ok(result)
    }

    fn from_account_info(a: &AccountInfo) -> Result<Self, ProgramError>
where {
        let ua = Self::safe_deserialize(&a.data.borrow_mut())
            .map_err(|_| SpriteManagerError::DataTypeMismatch)?;

        // Check that this is a `token-metadata` owned account.
        assert_owned_by(a, &crate::id(), SpriteManagerError::IncorrectOwner)?;

        Ok(ua)
    }
}
