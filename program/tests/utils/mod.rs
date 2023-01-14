mod assert;
mod edition_marker;
mod master_edition_v2;
mod metadata;

pub use assert::*;
pub use edition_marker::EditionMarker;
pub use master_edition_v2::MasterEditionV2;
pub use metadata::{assert_collection_size, Metadata};
pub use mpl_token_metadata::instruction;
use mpl_token_metadata::state::{Collection, CollectionDetails, EscrowAuthority};
use solana_program_test::*;
use solana_sdk::{
    account::Account, program_pack::Pack, pubkey::Pubkey, signature::Signer,
    signer::keypair::Keypair, system_instruction, transaction::Transaction,
};
use spl_token::state::Mint;
use sprite_manager::{instruction::*, pda::find_sprite_address};

pub const DEFAULT_COLLECTION_DETAILS: Option<CollectionDetails> =
    Some(CollectionDetails::V1 { size: 0 });

pub fn program_test() -> ProgramTest {
    let mut test = ProgramTest::new("sprite_manager", sprite_manager::id(), None);
    test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    test
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn get_mint(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Mint {
    let account = get_account(context, pubkey).await;
    Mint::unpack(&account.data).unwrap()
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}

pub async fn burn(
    context: &mut ProgramTestContext,
    metadata: Pubkey,
    owner: &Keypair,
    mint: Pubkey,
    token: Pubkey,
    edition: Pubkey,
    collection_metadata: Option<Pubkey>,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[instruction::burn_nft(
            mpl_token_metadata::ID,
            metadata,
            owner.pubkey(),
            mint,
            token,
            edition,
            spl_token::ID,
            collection_metadata,
        )],
        Some(&owner.pubkey()),
        &[owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn burn_edition(
    context: &mut ProgramTestContext,
    metadata: Pubkey,
    owner: &Keypair,
    print_edition_mint: Pubkey,
    master_edition_mint: Pubkey,
    print_edition_token: Pubkey,
    master_edition_token: Pubkey,
    master_edition: Pubkey,
    print_edition: Pubkey,
    edition_marker: Pubkey,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[instruction::burn_edition_nft(
            mpl_token_metadata::ID,
            metadata,
            owner.pubkey(),
            print_edition_mint,
            master_edition_mint,
            print_edition_token,
            master_edition_token,
            master_edition,
            print_edition,
            edition_marker,
            spl_token::ID,
        )],
        Some(&owner.pubkey()),
        &[owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    additional_signer: Option<&Keypair>,
) -> Result<(), BanksClientError> {
    let mut signing_keypairs = vec![&context.payer];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_token::instruction::mint_to(&spl_token::id(), mint, account, owner, &[], amount)
                .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                manager,
                freeze_authority,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

/// metadata is used as the Base NFT for the Trifle's Escrow account.
/// master_edition is the edition of the Base NFT
pub async fn create_sprite_account_helper(
    context: &mut ProgramTestContext,
    metadata: &Metadata,
    master_edition: &MasterEditionV2,
) -> (Pubkey, Pubkey) {
    let (sprite_addr, _) = find_sprite_address(&metadata.mint.pubkey());

    let (escrow_addr, _) = mpl_token_metadata::processor::find_escrow_account(
        &metadata.mint.pubkey(),
        &EscrowAuthority::Creator(sprite_addr.to_owned()),
    );

    let create_sprite_account_ix = create_sprite_account(
        &sprite_manager::id(),
        &escrow_addr,
        &metadata.pubkey,
        &metadata.mint.pubkey(),
        &metadata.token.pubkey(),
        &master_edition.pubkey,
        &sprite_addr,
        &context.payer.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_sprite_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    (sprite_addr, escrow_addr)
}

pub async fn create_nft(
    context: &mut ProgramTestContext,
    create_collection: bool,
    _freeze_authority: Option<Pubkey>,
) -> (Metadata, MasterEditionV2, Option<Metadata>) {
    if create_collection {
        let _payer_pubkey = context.payer.pubkey().to_owned();
        let collection = Metadata::new();
        let collection_master_edition = MasterEditionV2::new(&collection);
        collection
            .create_v2(
                context,
                "Collection".to_string(),
                "C".to_string(),
                "".to_string(),
                None,
                0,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        collection_master_edition
            .create_v3(context, Some(0))
            .await
            .unwrap();

        let metadata = Metadata::new();
        let master_edition = MasterEditionV2::new(&metadata);

        metadata
            .create_v2(
                context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                Some(Collection {
                    key: collection.mint.pubkey(),
                    verified: false,
                }),
                None,
            )
            .await
            .unwrap();

        master_edition.create(context, Some(1)).await.unwrap();

        let verify_collection_ix = mpl_token_metadata::instruction::verify_collection(
            mpl_token_metadata::id(),
            metadata.pubkey,
            context.payer.pubkey(),
            context.payer.pubkey(),
            collection.mint.pubkey(),
            collection.pubkey,
            collection_master_edition.pubkey,
            None,
        );
        let verify_collection_tx = Transaction::new_signed_with_payer(
            &[verify_collection_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context
            .banks_client
            .process_transaction(verify_collection_tx)
            .await
            .unwrap();
        (metadata, master_edition, Some(collection))
    } else {
        let metadata = Metadata::new();
        let master_edition = MasterEditionV2::new(&metadata);

        metadata
            .create_v2(
                context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        master_edition.create(context, Some(1)).await.unwrap();

        (metadata, master_edition, None)
    }
}

pub async fn create_sft(
    context: &mut ProgramTestContext,
    create_collection: bool,
    _freeze_authority: Option<Pubkey>,
) -> (Metadata, Option<Metadata>) {
    if create_collection {
        let _payer_pubkey = context.payer.pubkey().to_owned();
        let collection = Metadata::new();
        let collection_master_edition = MasterEditionV2::new(&collection);
        collection
            .create_v2(
                context,
                "Collection".to_string(),
                "C".to_string(),
                "".to_string(),
                None,
                0,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        collection_master_edition
            .create_v3(context, Some(0))
            .await
            .unwrap();

        let metadata = Metadata::new();

        metadata
            .create_fungible_v2(
                context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                Some(Collection {
                    key: collection.mint.pubkey(),
                    verified: false,
                }),
                None,
            )
            .await
            .unwrap();

        let verify_collection_ix = mpl_token_metadata::instruction::verify_collection(
            mpl_token_metadata::id(),
            metadata.pubkey,
            context.payer.pubkey(),
            context.payer.pubkey(),
            collection.mint.pubkey(),
            collection.pubkey,
            collection_master_edition.pubkey,
            None,
        );
        let verify_collection_tx = Transaction::new_signed_with_payer(
            &[verify_collection_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context
            .banks_client
            .process_transaction(verify_collection_tx)
            .await
            .unwrap();
        (metadata, Some(collection))
    } else {
        let metadata = Metadata::new();

        metadata
            .create_fungible_v2(
                context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        (metadata, None)
    }
}
