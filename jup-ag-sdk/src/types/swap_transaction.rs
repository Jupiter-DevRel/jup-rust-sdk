use serde::{Deserialize, Serialize};

use super::QuoteResponse;

/// SwapRequest is a struct that represents the request body for the swap transaction.
///
/// user SwapRequest::new() and the fluent setters to configure parameters.
///
/// [Official API docs](https://dev.jup.ag/docs/api/swap-api/swap)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    /// Rquired. The public key of the user initiating the swap.
    pub user_public_key: String,

    /// Allow a custom payer to pay for the transaction
    pub payer: String,

    /// Automatically wrap/unwrap native SOL to/from WSOL Default (true)
    /// When true, uses SOL and unwraps WSOL post-swap.
    /// When false, uses WSOL only and leaves it wrapped.
    /// Ignored if `destination_token_account` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_and_unwrap_sol: Option<bool>,

    /// Enables use of shared intermediate token accounts.
    /// Helps simplify swaps that use complex routing.
    /// Default: decided by routing engine
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_shared_accounts: Option<bool>,

    /// Associated Token account (must be input/output mint) to collect fees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,

    /// Tracking key to identify integrator or user swaps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_account: Option<String>,

    /// Optional prioritization fee configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritization_fee_lamports: Option<PrioritizationFeeLamports>,

    /// Build a legacy transaction instead of a versioned one.
    /// Should be consistent with the `/quote` response.
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_legacy_transaction: Option<bool>,

    /// Public key of a token account that will be used to receive the token out of the swap
    /// If not provided, the signer's token account will be used
    /// If provided, we assume that the token account is already initialized
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_token_account: Option<String>,

    /// When enabled, it will do a swap simulation to get the compute unit used and set it in ComputeBudget's compute unit limit
    /// This incurs one extra RPC call to simulate this
    /// We recommend to enable this to estimate compute unit correctly and reduce priority fees needed or have higher chance to be included in a block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_compute_unit_limit: Option<bool>,

    /// When enabled, it will not do any additional RPC calls to check on required accounts
    /// Enable it only when you already setup all the accounts needed for the trasaction, like wrapping or unwrapping sol, or destination account is already created
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_user_account_rpc_calls: Option<bool>,

    /// When enabled, it estimates slippage and apply it in the swap transaction directly, overwriting the slippageBps parameter in the quote response.
    /// Used together with dynamicSlippage in /quote, otherwise the slippage used will be the one in the /quote's slippageBps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dyanmic_slippage: Option<bool>,

    /// To use an exact compute unit price to calculate priority fee
    /// computeUnitLimit (1400000) * computeUnitPriceMicroLamports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_unit_price_micro_lamports: Option<u64>,
    /// Pass in the number of slots we want the transaction to be valid for
    /// Example: If you pass in 10 slots, the transaction will be valid for ~400ms * 10 = approximately 4 seconds before it expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockhash_slots_to_expiry: Option<u64>,
    pub quote_response: QuoteResponse,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrioritizationFeeLamports {
    pub jito_tip_lamports: Option<u64>,
    pub priority_level_with_max_lamports: PriorityLevelWithMaxLamports,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityLevelWithMaxLamports {
    pub max_lamports: u32,
    pub priority_level: PriorityLevel,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PriorityLevel {
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: u64,
}

impl SwapRequest {
    /// Creates a new `SwapRequest` from a user public key and quote response.
    ///
    /// # Arguments
    /// * `input_wallet` - The user's public key as a string.
    /// * `payer` - payer to pay for the transaction
    /// * `quote` - The `QuoteResponse` obtained from a quoting endpoint.
    ///
    /// # Returns
    /// A `SwapRequest` instance with default `None` values for all optional fields.
    ///
    /// # Example
    /// ```
    /// let payload = SwapRequest::new("YourPubKey...", quote);
    /// ```
    pub fn new(
        input_wallet: impl Into<String>,
        payer: impl Into<String>,
        quote: QuoteResponse,
    ) -> Self {
        Self {
            user_public_key: input_wallet.into(),
            payer: payer.into(),
            wrap_and_unwrap_sol: None,
            use_shared_accounts: None,
            fee_account: None,
            tracking_account: None,
            prioritization_fee_lamports: None,
            as_legacy_transaction: None,
            destination_token_account: None,
            dynamic_compute_unit_limit: None,
            skip_user_account_rpc_calls: None,
            dyanmic_slippage: None,
            compute_unit_price_micro_lamports: None,
            blockhash_slots_to_expiry: None,
            quote_response: quote,
        }
    }

    /// Sets whether to wrap or unwrap native SOL.
    ///
    /// If `true`, native SOL will be wrapped into WSOL before the swap
    /// and unwrapped afterward. This is the default behavior.
    pub fn wrap_and_unwrap_sol(mut self, wrap: bool) -> Self {
        self.wrap_and_unwrap_sol = Some(wrap);
        self
    }

    /// Sets whether to use shared intermediate token accounts.
    ///
    /// This can reduce transaction size and complexity for some swap routes.
    pub fn use_shared_accounts(mut self, shared: bool) -> Self {
        self.use_shared_accounts = Some(shared);
        self
    }

    /// Set An token account that will be used to collect fees
    ///
    /// The mint of the token account can only be either the input or output mint of the swap
    pub fn fee_account(mut self, account: String) -> Self {
        self.fee_account = Some(account);
        self
    }

    /// Specify any public key that belongs to you to track the transactions
    ///
    /// Useful for integrators to get all the swap transactions from this public key. Query the data using a block explorer like Solscan/SolanaFM or query like Dune/Flipside
    pub fn tracking_account(mut self, account: String) -> Self {
        self.tracking_account = Some(account);
        self
    }

    /// Set prioritization fee lamports
    pub fn prioritization_fee_jito_tip(mut self, fee: u64) -> Self {
        self.prioritization_fee_lamports = Some(PrioritizationFeeLamports {
            jito_tip_lamports: Some(fee),
            priority_level_with_max_lamports: PriorityLevelWithMaxLamports {
                max_lamports: 0,
                priority_level: PriorityLevel::Medium,
            },
        });
        self
    }

    /// set prioritization config
    pub fn prioritization_fee_config(
        mut self,
        jito_tip: Option<u64>,
        max_lamports: u32,
        priority_level: PriorityLevel,
    ) -> Self {
        self.prioritization_fee_lamports = Some(PrioritizationFeeLamports {
            jito_tip_lamports: jito_tip,
            priority_level_with_max_lamports: PriorityLevelWithMaxLamports {
                max_lamports,
                priority_level,
            },
        });
        self
    }

    /// Forces the transaction to be built as a legacy (non-versioned) transaction.
    pub fn as_legacy_transaction(mut self, legacy: bool) -> Self {
        self.as_legacy_transaction = Some(legacy);
        self
    }

    /// Sets a specific destination token account for the swap output.
    ///
    /// If not set, the user's associated token account will be used.
    pub fn destination_token_account(mut self, account: String) -> Self {
        self.destination_token_account = Some(account);
        self
    }

    /// Enables simulation-based estimation of compute unit usage.
    ///
    /// This helps optimize compute budget usage and reduce priority fees. one extra RPC call
    pub fn dynamic_compute_unit_limit(mut self, limit: bool) -> Self {
        self.dynamic_compute_unit_limit = Some(limit);
        self
    }

    /// Skips account-checking RPC calls.
    ///
    /// Enable only if you have pre-configured all token accounts and SOL wrapping/unwrapping.
    pub fn skip_user_account_rpc_calls(mut self, skip: bool) -> Self {
        self.skip_user_account_rpc_calls = Some(skip);
        self
    }

    /// Enables dynamic slippage estimation.
    ///
    /// If enabled, slippage will be recalculated at swap-time instead of using a fixed value.
    pub fn dyanmic_slippage(mut self, dynamic: bool) -> Self {
        self.dyanmic_slippage = Some(dynamic);
        self
    }

    /// Sets a fixed compute unit price in micro-lamports for fee calculation.
    pub fn compute_unit_price_micro_lamports(mut self, price: u64) -> Self {
        self.compute_unit_price_micro_lamports = Some(price);
        self
    }

    /// Sets the number of slots until the transaction expires.
    ///
    /// 1 slot ≈ 400ms. For example, 10 slots ≈ 4 seconds.
    pub fn blockhash_slots_to_expiry(mut self, slots: u64) -> Self {
        self.blockhash_slots_to_expiry = Some(slots);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub program_id: String,
    pub accounts: Vec<AccountMeta>,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructions {
    pub other_instructions: Option<Vec<Instruction>>,
    pub compute_budget_instructions: Option<Vec<Instruction>>,
    pub setup_instructions: Vec<Instruction>,
    pub swap_instruction: Instruction,
    pub cleanup_instruction: Option<Instruction>,
    pub address_lookup_table_addresses: Vec<String>,
}
