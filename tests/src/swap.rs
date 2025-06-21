#[cfg(test)]
mod swap_tests {
    use jup_ag_sdk::{
        JupiterClient,
        types::{DexEnum, QuoteGetSwapModeEnum, QuoteRequest, SwapRequest},
    };

    use crate::common::{
        BASE_URL, DEFAULT_SLIPPAGE_BPS, JUP_MINT, SOL_MINT, TEST_AMOUNT, TEST_USER_PUBKEY,
        create_test_client,
    };

    fn create_default_quote_request() -> QuoteRequest {
        QuoteRequest::new(SOL_MINT, JUP_MINT, TEST_AMOUNT)
            .slippage_bps(DEFAULT_SLIPPAGE_BPS)
            .swap_mode(QuoteGetSwapModeEnum::ExactOut)
    }

    #[test]
    fn test_jupiter_client_creation() {
        let client = create_test_client();
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn test_quote_request_builder_methods() {
        let request = QuoteRequest::new(SOL_MINT, JUP_MINT, TEST_AMOUNT)
            .slippage_bps(DEFAULT_SLIPPAGE_BPS)
            .swap_mode(QuoteGetSwapModeEnum::ExactOut)
            .dexes(vec![DexEnum::OrcaV1, DexEnum::MeteoraDlmm])
            .exclude_dexes(vec![DexEnum::Raydium])
            .restrict_intermediate_tokens(false)
            .only_direct_routes(true)
            .as_legacy_transaction(false)
            .platform_fee_bps(10);

        assert_eq!(request.input_mint, SOL_MINT, "input mint should match");
        assert_eq!(request.output_mint, JUP_MINT, "output mint should match");
        assert_eq!(request.amount, TEST_AMOUNT, "amount should match");

        assert_eq!(
            request.slippage_bps,
            Some(DEFAULT_SLIPPAGE_BPS),
            "slippage_bps should match"
        );
        assert_eq!(
            request.platform_fee_bps,
            Some(10),
            "platform fee should match"
        );

        assert_eq!(
            request.dexes,
            Some(vec![DexEnum::OrcaV1, DexEnum::MeteoraDlmm]),
            "dexes should match"
        );
        assert_eq!(
            request.exclude_dexes,
            Some(vec![DexEnum::Raydium]),
            "excluded dexes should match"
        );

        assert_eq!(
            request.restrict_intermediate_tokens,
            Some(false),
            "intermediate token restriction should match"
        );
        assert_eq!(
            request.only_direct_routes,
            Some(true),
            "direct routes setting should match"
        );
        assert_eq!(
            request.as_legacy_transaction,
            Some(false),
            "legacy transaction setting should match"
        );
    }

    #[tokio::test]
    async fn test_get_quote_successful() {
        let client = create_test_client();
        let quote = create_default_quote_request();

        match client.get_quote(&quote).await {
            Ok(quote_res) => {
                assert_eq!(
                    quote_res.input_mint, quote.input_mint,
                    "input mint should match"
                );
                assert_eq!(
                    quote_res.output_mint, quote.output_mint,
                    "output mint should match"
                );
                assert_eq!(
                    quote_res.out_amount,
                    TEST_AMOUNT.to_string(),
                    "output amount should match"
                );
                assert_eq!(
                    quote_res.slippage_bps, DEFAULT_SLIPPAGE_BPS,
                    "slippage should match"
                );
            }
            Err(err) => panic!("Quote request should succeed, got error: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_get_quote_with_invalid_endpoint() {
        let invalid_client = JupiterClient::new("https://lite-api.jup.ag/invalid");
        let quote = create_default_quote_request();

        let result = invalid_client.get_quote(&quote).await;
        assert!(result.is_err(), "Quote with invalid endpoint should fail");
    }

    #[tokio::test]
    async fn test_get_quote_with_invalid_params() {
        let client = create_test_client();
        let invalid_quote = QuoteRequest::new(SOL_MINT, "", TEST_AMOUNT);

        let result = client.get_quote(&invalid_quote).await;
        assert!(result.is_err(), "Quote with empty output mint should fail");
    }

    #[tokio::test]
    async fn test_swap_request_builder() {
        let client = create_test_client();
        let quote = create_default_quote_request();

        match client.get_quote(&quote).await {
            Ok(quote_res) => {
                let swap = SwapRequest::new(TEST_USER_PUBKEY, TEST_USER_PUBKEY, quote_res);

                assert_eq!(
                    swap.user_public_key, TEST_USER_PUBKEY,
                    "user public key should match"
                );
                assert_eq!(
                    swap.quote_response.input_mint, SOL_MINT,
                    "input mint should match"
                );
                assert_eq!(
                    swap.quote_response.out_amount,
                    TEST_AMOUNT.to_string(),
                    "output amount should match"
                );
            }
            Err(err) => panic!(
                "Quote request should succeed for swap test, got error: {:?}",
                err
            ),
        }
    }

    #[tokio::test]
    async fn test_get_swap_transaction() {
        let client = create_test_client();

        let quote_res = match client.get_quote(&create_default_quote_request()).await {
            Ok(res) => res,
            Err(err) => panic!("Failed to get quote for swap test: {:?}", err),
        };

        let swap = SwapRequest::new(TEST_USER_PUBKEY, TEST_USER_PUBKEY, quote_res);

        match client.get_swap_transaction(&swap).await {
            Ok(swap_res) => {
                assert!(
                    !swap_res.swap_transaction.is_empty(),
                    "Swap transaction should not be empty"
                );
            }
            Err(err) => panic!("Failed to get swap transaction: {:?}", err),
        }
    }
}
