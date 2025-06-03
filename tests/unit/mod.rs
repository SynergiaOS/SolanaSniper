// Unit tests for individual modules

pub mod helius_tests;
pub mod jito_tests;
pub mod jupiter_tests;
pub mod mem0_tests;
pub mod strategies;

// Test constants
pub const TEST_SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const TEST_USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const TEST_AMOUNT_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
pub const TEST_SLIPPAGE_BPS: u32 = 50; // 0.5%
