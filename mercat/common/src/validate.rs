use crate::{
    account_create_transaction_file, all_unverified_tx_files, asset_transaction_file,
    confidential_transaction_file, errors::Error, get_asset_ids, get_user_ticker_from, load_object,
    load_tx_file, parse_tx_name, save_object, save_to_file, user_public_account_file,
    CTXInstruction, CoreTransaction, Instruction, COMMON_OBJECTS_DIR, LAST_VALIDATED_TX_ID_FILE,
    MEDIATOR_PUBLIC_ACCOUNT_FILE, OFF_CHAIN_DIR, ON_CHAIN_DIR,
};
use codec::{Decode, Encode};
use cryptography::mercat::{
    account::AccountValidator, asset::AssetValidator, transaction::TransactionValidator,
    AccountCreatorVerifier, AccountMemo, AssetTransactionVerifier, AssetTxState, JustifiedAssetTx,
    JustifiedTx, PubAccount, PubAccountTx, TransactionVerifier, TxState, TxSubstate,
};
use log::{debug, info};
use metrics::timing;
use rand::rngs::OsRng;
use std::{path::PathBuf, time::Instant};

fn load_all_unverified_and_ready(db_dir: PathBuf) -> Result<Vec<CoreTransaction>, Error> {
    all_unverified_tx_files(db_dir)?
        .into_iter()
        .map(|tx| parse_tx_name(tx))
        .map(|res| match res {
            Err(error) => Err(error),
            Ok((tx_id, user, state, tx_file_path)) => {
                load_tx_file(tx_id, user, state, tx_file_path)
            }
        })
        .filter(|res| res.is_err() || res.as_ref().unwrap().is_ready_for_validation())
        .collect()
}

pub fn validate_all_pending(db_dir: PathBuf) -> Result<(), Error> {
    let all_unverified_and_ready = load_all_unverified_and_ready(db_dir.clone())?;
    let mut last_tx_id: i32 = -1;
    debug!("----> Read all the unverified transactions!");
    // For each of them call the validate function and process as needed
    for tx in all_unverified_and_ready {
        match tx {
            CoreTransaction::IssueJustify {
                issue_tx,
                tx_id,
                mediator,
            } => {
                validate_asset_issuance(db_dir.clone(), issue_tx, mediator, tx_id)?;
                last_tx_id = std::cmp::max(last_tx_id, tx_id as i32);
            }
            CoreTransaction::TransferJustify {
                tx,
                tx_id,
                mediator,
            } => {
                validate_transaction(db_dir.clone(), tx, mediator, tx_id)?;
                last_tx_id = std::cmp::max(last_tx_id, tx_id as i32);
            }
            CoreTransaction::Account { account_tx, tx_id } => {
                validate_account(db_dir.clone(), account_tx.content.pub_account.id)?;
                last_tx_id = std::cmp::max(last_tx_id, tx_id as i32);
            }
            _ => {
                return Err(Error::TransactionIsNotReadyForValidation { tx });
            }
        }
    }

    // TODO capture the updated account from each of the above calls, then decide about the final state of the account

    save_to_file(
        db_dir,
        OFF_CHAIN_DIR,
        COMMON_OBJECTS_DIR,
        LAST_VALIDATED_TX_ID_FILE,
        &last_tx_id,
    )?;
    Ok(())
}

pub fn validate_asset_issuance(
    db_dir: PathBuf,
    asset_tx: JustifiedAssetTx,
    mediator: String,
    tx_id: u32,
) -> Result<(), Error> {
    let load_objects_timer = Instant::now();

    let issuer_account_id = asset_tx.content.content.account_id;
    let (issuer, ticker, _) = get_user_ticker_from(issuer_account_id, db_dir.clone())?;
    info!(
        "Validating asset issuance{{tx_id: {}, issuer: {}, ticker: {}, mediator: {}}}",
        tx_id, issuer, ticker, mediator
    );
    let mediator_account: AccountMemo = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &mediator,
        MEDIATOR_PUBLIC_ACCOUNT_FILE,
    )?;

    let issuer_account: PubAccount = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &issuer,
        &user_public_account_file(&ticker),
    )?;
    timing!(
        "validator.issuance.load_objects",
        load_objects_timer,
        Instant::now()
    );

    let validate_issuance_transaction_timer = Instant::now();

    let validator = AssetValidator {};
    let updated_issuer_account = validator
        .verify_asset_transaction(
            &asset_tx,
            issuer_account,
            &mediator_account.owner_enc_pub_key,
            &mediator_account.owner_sign_pub_key,
        )
        .map_err(|error| Error::LibraryError { error })?;

    timing!(
        "validator.issuance.transaction",
        validate_issuance_transaction_timer,
        Instant::now()
    );

    let save_objects_timer = Instant::now();
    // Save the transaction under the new state.
    let new_state = AssetTxState::Justification(TxSubstate::Validated);
    let instruction = Instruction {
        state: new_state,
        data: asset_tx.encode().to_vec(),
    };
    save_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &issuer,
        &asset_transaction_file(tx_id, &issuer, new_state),
        &instruction,
    )?;

    // Save the updated issuer account.
    save_object(
        db_dir,
        ON_CHAIN_DIR,
        &issuer,
        &user_public_account_file(&ticker),
        &updated_issuer_account,
    )?;

    timing!(
        "validator.issuance.save_objects",
        save_objects_timer,
        Instant::now()
    );

    Ok(())
}

pub fn validate_account(db_dir: PathBuf, account_id: u32) -> Result<(), Error> {
    // Load the user's public account.
    let load_objects_timer = Instant::now();

    let (user, ticker, tx_id) = get_user_ticker_from(account_id, db_dir.clone())?;
    info!(
        "Validating account{{id: {}, user: {}, ticker: {}}}",
        account_id, user, ticker
    );
    let user_account: PubAccountTx = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        COMMON_OBJECTS_DIR,
        &account_create_transaction_file(tx_id, &user, &ticker),
    )?;

    let valid_asset_ids = get_asset_ids(db_dir.clone())?;
    timing!(
        "validator.account.load_objects",
        load_objects_timer,
        Instant::now()
    );

    // Validate the account.
    let validate_account_timer = Instant::now();
    let account_validator = AccountValidator {};
    account_validator
        .verify(&user_account, &valid_asset_ids)
        .map_err(|error| Error::LibraryError { error })?;

    timing!("validator.account", validate_account_timer, Instant::now());

    // On success save the public account as validated.
    let save_objects_timer = Instant::now();
    save_object(
        db_dir,
        ON_CHAIN_DIR,
        &user,
        &user_public_account_file(&ticker),
        &user_account,
    )?;

    timing!(
        "validator.account.save_objects",
        save_objects_timer,
        Instant::now()
    );

    Ok(())
}

fn process_transaction(
    instruction: CTXInstruction,
    sender_pub_account: PubAccount,
    receiver_pub_account: PubAccount,
    mdtr_account: &AccountMemo,
) -> Result<(PubAccount, PubAccount), Error> {
    let mut rng = OsRng::default();
    let tx = JustifiedTx::decode(&mut &instruction.data[..]).unwrap();
    let validator = TransactionValidator {};
    let (updated_sender_account, updated_receiver_account) = validator
        .verify_transaction(
            &tx,
            sender_pub_account,
            receiver_pub_account,
            &mdtr_account.owner_sign_pub_key,
            &mut rng,
        )
        .map_err(|error| Error::LibraryError { error })?;

    Ok((updated_sender_account, updated_receiver_account))
}

pub fn validate_transaction(
    db_dir: PathBuf,
    tx: JustifiedTx,
    mediator: String,
    tx_id: u32,
) -> Result<(), Error> {
    let load_objects_timer = Instant::now();
    // Load the transaction, mediator's account, and issuer's public account.

    let (sender, _, _) = get_user_ticker_from(
        tx.content.content.init_data.content.memo.sndr_account_id,
        db_dir.clone(),
    )?;
    let (receiver, ticker, _) = get_user_ticker_from(
        tx.content.content.init_data.content.memo.rcvr_account_id,
        db_dir.clone(),
    )?;
    info!(
        "Validating asset transfer{{tx_id: {}, sender: {}, receiver: {}, ticker:{}, mediator: {}}}",
        tx_id, sender, receiver, ticker, mediator
    );
    let state = TxState::Justification(TxSubstate::Started);

    let mut instruction: CTXInstruction = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        COMMON_OBJECTS_DIR,
        &confidential_transaction_file(tx_id, &mediator, state),
    )?;

    let mediator_account: AccountMemo = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &mediator,
        MEDIATOR_PUBLIC_ACCOUNT_FILE,
    )?;

    let sender_account: PubAccount = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &sender,
        &user_public_account_file(&ticker),
    )?;

    let receiver_account: PubAccount = load_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &receiver,
        &user_public_account_file(&ticker),
    )?;

    timing!(
        "validator.issuance.load_objects",
        load_objects_timer,
        Instant::now()
    );

    let validate_transaction_timer = Instant::now();
    let (updated_sender_account, updated_receiver_account) = process_transaction(
        instruction.clone(),
        sender_account,
        receiver_account,
        &mediator_account,
    )?;

    timing!(
        "validator.transaction",
        validate_transaction_timer,
        Instant::now()
    );

    let save_objects_timer = Instant::now();
    // Save the transaction under the new state.
    instruction.state = TxState::Justification(TxSubstate::Validated);
    save_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        COMMON_OBJECTS_DIR,
        &confidential_transaction_file(tx_id, &sender, instruction.state),
        &instruction,
    )?;

    // Save the updated sender and receiver accounts.
    save_object(
        db_dir.clone(),
        ON_CHAIN_DIR,
        &sender,
        &user_public_account_file(&ticker),
        &updated_sender_account,
    )?;
    save_object(
        db_dir,
        ON_CHAIN_DIR,
        &receiver,
        &user_public_account_file(&ticker),
        &updated_receiver_account,
    )?;

    timing!(
        "validator.issuance.save_objects",
        save_objects_timer,
        Instant::now()
    );

    Ok(())
}
