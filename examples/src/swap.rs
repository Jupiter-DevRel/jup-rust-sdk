use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use dotenv::dotenv;
use jup_ag_sdk::{
    JupiterClient,
    types::{Instruction, QuoteGetSwapModeEnum, QuoteRequest, SwapRequest, SwapResponse},
};
use solana_sdk::{
    address_lookup_table::state::AddressLookupTable,
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta as SolanaAccountMeta, Instruction as SolanaInstruction},
    message::{
        AddressLookupTableAccount, VersionedMessage,
        v0::{self},
    },
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::{env, error::Error};

pub async fn swap() {
    let client = JupiterClient::new("https://lite-api.jup.ag");

    let quote = QuoteRequest::new(
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        1_000_000, // 1 USDC (6 decimals)
    )
    .swap_mode(QuoteGetSwapModeEnum::ExactOut); // Swap some SOL for exact 1 USDC

    let quote_res = client.get_quote(&quote).await.expect("Failed to get quote");

    let payload = SwapRequest::new(
        "input_your_wallet_address",
        "payer_wallet_address",
        quote_res,
    );
    let swap_res: SwapResponse = client
        .get_swap_transaction(&payload)
        .await
        .expect("Failed to get swap transaction");

    // Load .env variables into std::env
    dotenv().ok();

    // Read the variable
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set in .env");

    let key_bytes = bs58::decode(key)
        .into_vec()
        .expect("Failed to decode base58 private key");

    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");

    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=";
    let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

    let swap_tx_bytes = STANDARD
        .decode(swap_res.swap_transaction)
        .expect("Failed to decode base64 transaction");
    let mut tx: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();
    let message = tx.message.serialize();
    let signature = keypair.sign_message(&message);

    if tx.signatures.is_empty() {
        // If no signatures array exists (unlikely with Jupiter)
        tx.signatures.push(signature);
    } else {
        // Replace the first signature (fee payer)
        tx.signatures[0] = signature;
    };

    let signature = rpc_client.send_and_confirm_transaction(&tx).unwrap();

    println!("Transaction signature: {}", signature);
}

pub async fn swap_with_instructions() {
    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=";
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

    let client = JupiterClient::new("https://lite-api.jup.ag");

    // get quote
    let quote = QuoteRequest::new(
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        1_000_000, // 1 USDC (6 decimals)
    )
    .swap_mode(QuoteGetSwapModeEnum::ExactOut); // Swap some SOL for exact 1 USDC

    let quote_res = client.get_quote(&quote).await.expect("Failed to get quote");

    // get swap instructions
    let payload = SwapRequest::new(
        "EXBdeRCdiNChKyD7akt64n9HgSXEpUtpPEhmbnm4L6iH",
        "payer_wallet_address",
        quote_res,
    );

    let swap_instructions = client
        .get_swap_instructions(&payload)
        .await
        .expect("Failed to get swap instructions");

    // convert the swap instructions to Solana instructions type
    let mut instructions = vec![];

    if let Some(compute_instructions) = swap_instructions.compute_budget_instructions {
        for instr in compute_instructions {
            instructions.push(parse_instruction(&instr).unwrap());
        }
    }

    for instr in swap_instructions.setup_instructions {
        instructions.push(parse_instruction(&instr).unwrap());
    }

    instructions.push(parse_instruction(&swap_instructions.swap_instruction).unwrap());

    if let Some(cleanup_instr) = swap_instructions.cleanup_instruction {
        instructions.push(parse_instruction(&cleanup_instr).unwrap());
    }

    if let Some(other_instructions) = swap_instructions.other_instructions {
        for instr in other_instructions {
            instructions.push(parse_instruction(&instr).unwrap());
        }
    }

    // get address lookup tables
    let mut address_table_lookups = vec![];
    for alt_address in swap_instructions.address_lookup_table_addresses {
        let alt_pubkey = alt_address.parse::<Pubkey>().unwrap();
        let alt_account = rpc_client.get_account(&alt_pubkey).await.unwrap();
        let alt_state = AddressLookupTable::deserialize(&alt_account.data).unwrap();

        let address_table_account = AddressLookupTableAccount {
            key: alt_pubkey,
            addresses: alt_state.addresses.into_owned(),
        };

        address_table_lookups.push(address_table_account);
    }

    dotenv().ok();

    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set in .env");

    let key_bytes = bs58::decode(key)
        .into_vec()
        .expect("Failed to decode base58 private key");

    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("Failed to get blockhash");

    let message = v0::Message::try_compile(
        &keypair.pubkey(),
        &instructions,
        &address_table_lookups,
        recent_blockhash,
    )
    .unwrap();

    let versioned_message = VersionedMessage::V0(message);

    let tx = VersionedTransaction::try_new(versioned_message, &[&keypair]).unwrap();

    let signature = rpc_client.send_and_confirm_transaction(&tx).await.unwrap();
    println!("Tx sent with signature: {}", signature);
}

// Helper function
fn parse_instruction(instr: &Instruction) -> Result<SolanaInstruction, Box<dyn Error>> {
    let program_id = instr
        .program_id
        .parse::<Pubkey>()
        .map_err(|e| format!("Invalid program_id pubkey: {}", e))?;

    let accounts: Vec<SolanaAccountMeta> = instr
        .accounts
        .iter()
        .map(|a| {
            let pubkey = a
                .pubkey
                .parse::<Pubkey>()
                .map_err(|e| format!("Invalid account pubkey: {}", e))?;
            Ok(SolanaAccountMeta {
                pubkey,
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
        })
        .collect::<Result<Vec<_>, String>>()?; // Explicit `Result` type;

    let data = STANDARD
        .decode(&instr.data)
        .map_err(|e| format!("Base64 decoding error in instruction data: {}", e))?;

    Ok(SolanaInstruction {
        program_id,
        accounts,
        data,
    })
}
