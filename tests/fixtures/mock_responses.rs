// Mock API responses for testing
// Provides realistic mock responses for all integrated APIs

use serde_json::json;
use std::collections::HashMap;

/// Jupiter API mock responses
pub struct JupiterMockResponses;

impl JupiterMockResponses {
    pub fn successful_quote() -> serde_json::Value {
        json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "inAmount": "1000000",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "outAmount": "950000",
            "otherAmountThreshold": "940000",
            "swapMode": "ExactIn",
            "slippageBps": 50,
            "platformFee": null,
            "priceImpactPct": "0.1",
            "routePlan": [
                {
                    "swapInfo": {
                        "ammKey": "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
                        "label": "Raydium",
                        "inputMint": "So11111111111111111111111111111111111111112",
                        "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "inAmount": "1000000",
                        "outAmount": "950000",
                        "feeAmount": "2500",
                        "feeMint": "So11111111111111111111111111111111111111112"
                    },
                    "percent": 100
                }
            ],
            "contextSlot": 123456789,
            "timeTaken": 0.05
        })
    }

    pub fn high_slippage_quote() -> serde_json::Value {
        json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "inAmount": "1000000",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "outAmount": "850000",
            "otherAmountThreshold": "800000",
            "swapMode": "ExactIn",
            "slippageBps": 50,
            "platformFee": null,
            "priceImpactPct": "15.0",
            "routePlan": [],
            "contextSlot": 123456789,
            "timeTaken": 0.12
        })
    }

    pub fn no_route_error() -> serde_json::Value {
        json!({
            "error": "No routes found",
            "errorCode": "NO_ROUTES_FOUND",
            "message": "No routes found for the given input and output mints"
        })
    }

    pub fn successful_swap() -> serde_json::Value {
        json!({
            "swapTransaction": "base64_encoded_transaction_data",
            "lastValidBlockHeight": 123456790,
            "prioritizationFeeLamports": 5000
        })
    }
}

/// Helius API mock responses
pub struct HeliusMockResponses;

impl HeliusMockResponses {
    pub fn account_info() -> serde_json::Value {
        json!({
            "context": {
                "slot": 123456789
            },
            "value": {
                "data": ["base64_encoded_account_data", "base64"],
                "executable": false,
                "lamports": 1000000000,
                "owner": "11111111111111111111111111111111",
                "rentEpoch": 361
            }
        })
    }

    pub fn token_metadata() -> serde_json::Value {
        json!({
            "interface": "V1_NFT",
            "id": "So11111111111111111111111111111111111111112",
            "content": {
                "metadata": {
                    "name": "Wrapped SOL",
                    "symbol": "SOL",
                    "description": "Wrapped Solana token",
                    "image": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
                },
                "links": {
                    "external_url": "https://solana.com/"
                }
            },
            "authorities": [
                {
                    "address": "So11111111111111111111111111111111111111112",
                    "scopes": ["full"]
                }
            ],
            "compression": {
                "eligible": false,
                "compressed": false
            },
            "grouping": [],
            "royalty": {
                "royalty_model": "creators",
                "target": null,
                "percent": 0,
                "basis_points": 0,
                "primary_sale_happened": true,
                "locked": false
            },
            "creators": [],
            "ownership": {
                "frozen": false,
                "delegated": false,
                "delegate": null,
                "ownership_model": "single",
                "owner": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
            },
            "supply": {
                "print_max_supply": 0,
                "print_current_supply": 0,
                "edition_nonce": null
            },
            "mutable": true,
            "burnt": false
        })
    }

    pub fn webhook_notification() -> serde_json::Value {
        json!({
            "accountData": [
                {
                    "account": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                    "nativeBalanceChange": 1000000,
                    "tokenBalanceChanges": [
                        {
                            "mint": "So11111111111111111111111111111111111111112",
                            "rawTokenAmount": {
                                "tokenAmount": "1000000",
                                "decimals": 9
                            },
                            "tokenAccount": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                            "userAccount": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
                        }
                    ]
                }
            ],
            "blockTime": 1640995200,
            "slot": 123456789,
            "txnSignature": "5j7s8K9mN2pQ3rT4uV5wX6yZ7a8B9c0D1e2F3g4H5i6J7k8L9m0N1o2P3q4R5s6T7u8V9w0X1y2Z3a4B5c6D7e8F",
            "type": "TRANSFER"
        })
    }

    pub fn enhanced_transaction() -> serde_json::Value {
        json!({
            "description": "Swap 0.001 SOL for USDC on Raydium",
            "type": "SWAP",
            "source": "RAYDIUM",
            "fee": 2500,
            "feePayer": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "signature": "5j7s8K9mN2pQ3rT4uV5wX6yZ7a8B9c0D1e2F3g4H5i6J7k8L9m0N1o2P3q4R5s6T7u8V9w0X1y2Z3a4B5c6D7e8F",
            "slot": 123456789,
            "timestamp": 1640995200,
            "tokenTransfers": [
                {
                    "fromTokenAccount": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                    "toTokenAccount": "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
                    "fromUserAccount": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                    "toUserAccount": "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
                    "tokenAmount": 1000000,
                    "mint": "So11111111111111111111111111111111111111112"
                }
            ],
            "nativeTransfers": [],
            "accountData": [],
            "transactionError": null,
            "instructions": [
                {
                    "accounts": [
                        "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                        "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2"
                    ],
                    "data": "base64_instruction_data",
                    "programId": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
                    "innerInstructions": []
                }
            ],
            "events": {}
        })
    }
}

/// Mem0 API mock responses
pub struct Mem0MockResponses;

impl Mem0MockResponses {
    pub fn successful_memory_creation() -> serde_json::Value {
        json!({
            "id": "mem_123456789",
            "content": "Token analysis: High liquidity, low risk score",
            "metadata": {
                "token_address": "So11111111111111111111111111111111111111112",
                "risk_score": "0.2",
                "strategy": "microbot",
                "outcome": "SUCCESS"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "user_id": "sniperbot-test"
        })
    }

    pub fn memory_search_results() -> serde_json::Value {
        json!({
            "memories": [
                {
                    "id": "mem_123456789",
                    "content": "Token analysis: High liquidity, low risk score",
                    "metadata": {
                        "token_address": "So11111111111111111111111111111111111111112",
                        "risk_score": "0.2",
                        "strategy": "microbot",
                        "outcome": "SUCCESS"
                    },
                    "created_at": "2024-01-01T00:00:00Z",
                    "score": 0.95
                },
                {
                    "id": "mem_987654321",
                    "content": "Previous trade on this token resulted in 5% profit",
                    "metadata": {
                        "token_address": "So11111111111111111111111111111111111111112",
                        "pnl": "0.05",
                        "strategy": "microbot",
                        "outcome": "SUCCESS"
                    },
                    "created_at": "2023-12-31T23:00:00Z",
                    "score": 0.87
                }
            ],
            "total": 2,
            "page": 1,
            "per_page": 10
        })
    }

    pub fn memory_deletion_success() -> serde_json::Value {
        json!({
            "message": "Memory deleted successfully",
            "deleted_id": "mem_123456789"
        })
    }

    pub fn rate_limit_error() -> serde_json::Value {
        json!({
            "error": "Rate limit exceeded",
            "error_code": "RATE_LIMIT_EXCEEDED",
            "message": "Too many requests. Please try again later.",
            "retry_after": 60
        })
    }
}

/// Jito API mock responses
pub struct JitoMockResponses;

impl JitoMockResponses {
    pub fn successful_bundle_submission() -> serde_json::Value {
        json!({
            "bundle_id": "bundle_123456789",
            "status": "submitted",
            "slot": 123456790,
            "transactions": [
                "5j7s8K9mN2pQ3rT4uV5wX6yZ7a8B9c0D1e2F3g4H5i6J7k8L9m0N1o2P3q4R5s6T7u8V9w0X1y2Z3a4B5c6D7e8F"
            ],
            "tip_amount": 25000,
            "submitted_at": "2024-01-01T00:00:00Z"
        })
    }

    pub fn bundle_confirmation() -> serde_json::Value {
        json!({
            "bundle_id": "bundle_123456789",
            "status": "confirmed",
            "slot": 123456790,
            "block_hash": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "confirmation_time": "2024-01-01T00:00:05Z",
            "transactions": [
                {
                    "signature": "5j7s8K9mN2pQ3rT4uV5wX6yZ7a8B9c0D1e2F3g4H5i6J7k8L9m0N1o2P3q4R5s6T7u8V9w0X1y2Z3a4B5c6D7e8F",
                    "status": "confirmed",
                    "compute_units_consumed": 150000
                }
            ]
        })
    }

    pub fn bundle_rejection() -> serde_json::Value {
        json!({
            "bundle_id": "bundle_123456789",
            "status": "rejected",
            "reason": "Insufficient tip amount",
            "error_code": "INSUFFICIENT_TIP",
            "suggested_tip": 50000,
            "rejected_at": "2024-01-01T00:00:02Z"
        })
    }

    pub fn bundle_expiration() -> serde_json::Value {
        json!({
            "bundle_id": "bundle_123456789",
            "status": "expired",
            "reason": "Bundle not included within timeout period",
            "error_code": "BUNDLE_EXPIRED",
            "expired_at": "2024-01-01T00:01:00Z"
        })
    }

    pub fn tip_stream_update() -> serde_json::Value {
        json!({
            "slot": 123456790,
            "tips": [
                {
                    "account": "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
                    "amount": 25000
                },
                {
                    "account": "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
                    "amount": 15000
                }
            ],
            "timestamp": "2024-01-01T00:00:00Z"
        })
    }
}

/// Error response generators
pub struct ErrorResponses;

impl ErrorResponses {
    pub fn network_timeout() -> serde_json::Value {
        json!({
            "error": "Network timeout",
            "error_code": "NETWORK_TIMEOUT",
            "message": "Request timed out after 30 seconds",
            "retry_after": 5
        })
    }

    pub fn invalid_request() -> serde_json::Value {
        json!({
            "error": "Invalid request",
            "error_code": "INVALID_REQUEST",
            "message": "Missing required parameter: inputMint",
            "details": {
                "missing_fields": ["inputMint"],
                "provided_fields": ["outputMint", "amount"]
            }
        })
    }

    pub fn service_unavailable() -> serde_json::Value {
        json!({
            "error": "Service unavailable",
            "error_code": "SERVICE_UNAVAILABLE",
            "message": "Service is temporarily unavailable. Please try again later.",
            "retry_after": 300
        })
    }

    pub fn authentication_failed() -> serde_json::Value {
        json!({
            "error": "Authentication failed",
            "error_code": "AUTH_FAILED",
            "message": "Invalid API key or token",
            "details": {
                "provided_key_format": "valid",
                "key_status": "expired"
            }
        })
    }
}

/// Helper functions for creating mock responses
pub fn create_custom_jupiter_quote(
    input_mint: &str,
    output_mint: &str,
    in_amount: &str,
    out_amount: &str,
    price_impact: &str,
) -> serde_json::Value {
    json!({
        "inputMint": input_mint,
        "inAmount": in_amount,
        "outputMint": output_mint,
        "outAmount": out_amount,
        "otherAmountThreshold": out_amount,
        "swapMode": "ExactIn",
        "slippageBps": 50,
        "platformFee": null,
        "priceImpactPct": price_impact,
        "routePlan": [],
        "contextSlot": 123456789,
        "timeTaken": 0.05
    })
}

pub fn create_custom_mem0_memory(
    content: &str,
    metadata: HashMap<String, String>,
) -> serde_json::Value {
    json!({
        "id": "mem_custom_123",
        "content": content,
        "metadata": metadata,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "user_id": "sniperbot-test"
    })
}

pub fn create_custom_jito_bundle(
    bundle_id: &str,
    status: &str,
    tip_amount: u64,
) -> serde_json::Value {
    json!({
        "bundle_id": bundle_id,
        "status": status,
        "slot": 123456790,
        "tip_amount": tip_amount,
        "submitted_at": "2024-01-01T00:00:00Z"
    })
}
