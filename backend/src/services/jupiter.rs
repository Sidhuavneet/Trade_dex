// Jupiter API service module
// Price API V3: https://lite-api.jup.ag/price/v3
// Swap API V6: https://quote-api.jup.ag/v6

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct JupiterService {
    price_api_url: String,
    swap_api_url: String,
}

// Jupiter Price API V3 response format
// Response is a map keyed by token mint address
#[derive(Debug, Deserialize)]
pub struct PriceDataV3 {
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "blockId")]
    pub block_id: Option<u64>,
    pub decimals: Option<u8>,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: Option<f64>,
}

// Jupiter Swap API V6 quote response format
#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub fee_bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

impl JupiterService {
    pub fn new() -> Result<Self> {
        // Hardcoded Jupiter API URLs (not from .env)
        // Price API V3: https://lite-api.jup.ag/price/v3
        // Swap API V6: https://quote-api.jup.ag/v6
        Ok(Self {
            price_api_url: "https://lite-api.jup.ag/price/v3".to_string(),
            swap_api_url: "https://quote-api.jup.ag/v6".to_string(),
        })
    }

    /// Get price for a token pair (Jupiter Price API V3)
    /// Uses: https://lite-api.jup.ag/price/v3?ids={token_mint}
    /// For non-USDC quote tokens, calculates price as base_usd_price / quote_usd_price
    pub async fn get_price(&self, base_mint: &str, quote_mint: &str) -> Result<f64> {
        let client = reqwest::Client::new();
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        
        // If quote is USDC, get base price in USD directly
        if quote_mint == usdc_mint {
            let url = format!("{}?ids={}", self.price_api_url, base_mint);
            let response: HashMap<String, PriceDataV3> = client
                .get(&url)
                .send()
                .await?
                .json()
                .await?;
            
            let price_data = response
                .get(base_mint)
                .ok_or_else(|| anyhow::anyhow!("Price data not found for {}", base_mint))?;
            
            Ok(price_data.usd_price)
        } else {
            // For non-USDC quote tokens, get both prices in USD separately and calculate ratio
            // Make two separate API calls to avoid issues with comma-separated IDs
            let base_url = format!("{}?ids={}", self.price_api_url, base_mint);
            let quote_url = format!("{}?ids={}", self.price_api_url, quote_mint);
            
            // Fetch base token price
            let base_response_result = client
                .get(&base_url)
                .send()
                .await;
            
            let base_response = base_response_result
                .map_err(|e| anyhow::anyhow!("Failed to fetch base token price: {}", e))?;
            
            let base_response_json: HashMap<String, PriceDataV3> = base_response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse base token price response: {}", e))?;
            
            let base_price_data = base_response_json
                .get(base_mint)
                .ok_or_else(|| anyhow::anyhow!("Price data not found for base token {}", base_mint))?;
            
            // Fetch quote token price
            let quote_response_result = client
                .get(&quote_url)
                .send()
                .await;
            
            let quote_response = quote_response_result
                .map_err(|e| anyhow::anyhow!("Failed to fetch quote token price: {}", e))?;
            
            let quote_response_json: HashMap<String, PriceDataV3> = quote_response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse quote token price response: {}", e))?;
            
            let quote_price_data = quote_response_json
                .get(quote_mint)
                .ok_or_else(|| anyhow::anyhow!("Price data not found for quote token {}", quote_mint))?;
            
            // Calculate price as base_usd_price / quote_usd_price
            if quote_price_data.usd_price > 0.0 {
                Ok(base_price_data.usd_price / quote_price_data.usd_price)
            } else {
                Err(anyhow::anyhow!("Quote token price is zero or invalid"))
            }
        }
    }

    /// Get quote for a swap (Jupiter Swap API V6)
    /// Uses: https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.swap_api_url, input_mint, output_mint, amount, slippage_bps
        );
        
        let response: QuoteResponse = client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Get price for SOL/USDC pair
    pub async fn get_sol_usdc_price(&self) -> Result<f64> {
        // SOL mint: So11111111111111111111111111111111111111112
        // USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        
        self.get_price(sol_mint, usdc_mint).await
    }
}
