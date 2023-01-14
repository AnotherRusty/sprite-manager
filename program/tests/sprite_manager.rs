#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use sprite_manager::instruction::*;
use utils::*;

mod sprite_manager_test {
    use solana_program::borsh::try_from_slice_unchecked;
    use sprite_manager::state::SpriteAccount;

    use super::*;

    #[tokio::test]
    async fn test_happy_path() {
        let mut context = program_test().start_with_context().await;

        let payer_pubkey = context.payer.pubkey().to_owned();
        let (metadata, master_edition, test_collection) =
            create_nft(&mut context, true, Some(payer_pubkey)).await;
        let _test_collection = test_collection.expect("test collection should exist");

        let (sprite_manager_addr, escrow_addr) =
            create_sprite_account_helper(&mut context, &metadata, &master_edition).await;

        let sprite_manager_account = context
            .banks_client
            .get_account(sprite_manager_addr)
            .await
            .expect("sprite account should exist")
            .expect("sprite account should exist");

        let sprite_manager_account_data: SpriteAccount =
            try_from_slice_unchecked(&sprite_manager_account.data).expect("should deserialize");
        println!("sprite_account: {:#?}", sprite_manager_account_data);

        // Build the sprite
        let (sprite_metadata, _sprite_master_edition, _) =
            create_nft(&mut context, false, None).await;

        let sprite_token_account = spl_associated_token_account::get_associated_token_address(
            &escrow_addr,
            &sprite_metadata.mint.pubkey(),
        );

        let store_ix = store_sprite(
            &sprite_manager::ID,
            &escrow_addr,
            &metadata.mint.pubkey(),
            &sprite_metadata.mint.pubkey(),
            &sprite_metadata.token.pubkey(),
            &sprite_token_account,
            &context.payer.pubkey(),
            &sprite_manager_addr,
            "test".to_string(),
            "a test".to_string(),
            vec![],
            vec![],
            vec!["test".to_string()],
        );

        let store_tx = Transaction::new_signed_with_payer(
            &[store_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(store_tx)
            .await
            .expect("storing the sprite should succeed");

        let sprite_manager_account = context
            .banks_client
            .get_account(sprite_manager_addr)
            .await
            .expect("sprite account should exist")
            .expect("sprite account should exist");

        let sprite_manager_account_data: SpriteAccount =
            try_from_slice_unchecked(&sprite_manager_account.data).expect("should deserialize");
        println!("sprite_account: {:#?}", sprite_manager_account_data);
    }
}
