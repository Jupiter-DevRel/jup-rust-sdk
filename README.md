# Jup-Rust-SDK

> A Rust SDK to interact with the [Jupiter Aggregator](https://jup.ag) APIs.  
> Easily fetch token quotes and execute swaps programmatically from your Rust applications.

## Installation 🛠️

```bash
cargo add jup-ag-sdk
```

or Add this to your `Cargo.toml`:

```toml
[dependencies]
jup-ag-sdk = "1.0.4"
```

## Features

- Complete API Coverage - All Jupiter APIs included Ultra, Swap, Trigger, Recurring, Token and Price
- Strongly typed – Full Rust structs for all request/response types
- Composable builders – Chainable methods to customize request payloads (e.g. taker, referral, fee, excluded routers)

## Usage

Below is a simple example that shows how to fetch and execute an Ultra order with the SDK. For detailed example checkout [`examples/`](https://github.com/Jupiter-DevRel/jup-rust-sdk/tree/main/examples)

```rust
use jup_ag_sdk::{
    JupiterClient,
    types::{UltraExecuteOrderRequest, UltraOrderRequest},
};

#[tokio::main]
async fn main() {
    // Initialize the Jupiter client with the Lite API endpoint
    let client = JupiterClient::new("https://lite-api.jup.ag");

    // Example: swap: 10 USDC (6 decimals) to SOL
    let input_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let output_token = "So11111111111111111111111111111111111111112"; // SOL
    let amount = 10_000_000; // 10 USDC (in micro units)

    // Create an order request and specify the taker's wallet address
    let order_request = UltraOrderRequest::new(input_token, output_token, amount)
        .add_taker("your taker wallet address");

    // Fetch the unsigned transaction and request ID from Jupiter
    let order_response = client
        .get_ultra_order(&order_request)
        .await
        .expect("Failed to fetch ultra order");

    let unsigned_tx_base64 = order_response
        .transaction
        .expect("No transaction found in ultra order response");

    // sign the transaction. Checkout examples/src/lib.rs on how to sign the transaction

    // execute the signed transaction
    let execute_request = UltraExecuteOrderRequest {
        signed_transaction: signed_tx_base64,
        request_id: quote.request_id,
    };

    // Send the signed transaction to Jupiter for execution
    let execute_response = client
        .ultra_execute_order(&execute_request)
        .await
        .expect("Failed to execute transaction");

    // Print the transaction signature
    let tx_signature = execute_response
        .signature
        .expect("No signature found in execution response");

    println!("✅ Transaction submitted: {}", tx_signature);
}
```

## Support

- [API Documentation](https://dev.jup.ag/)
- [Discord](https://discord.gg/jup)
- [crates.io](https://crates.io/crates/jup-ag-sdk)

## Local

```bash
git clone https://github.com/Jupiter-DevRel/jup-rust-sdk
cd jup-rust-sdk
cargo build
```

open to contributions and suggestions.

## License

MIT License
