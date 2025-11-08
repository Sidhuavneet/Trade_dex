// Pair symbol to mint address mapping utility

use std::collections::HashMap;

/// Map symbol to mint address
pub fn symbol_to_mint(symbol: &str) -> Option<&str> {
    match symbol {
        "SOL" => Some("So11111111111111111111111111111111111111112"),
        "USDC" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "USDT" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        "BONK" => Some("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        "JUP" => Some("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
        "WIF" => Some("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"),
        "RAY" => Some("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
        _ => None,
    }
}

/// Parse pair string (e.g., "SOL/USDC") into base and quote symbols
pub fn parse_pair(pair: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = pair.split('/').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Get mint addresses for a pair
pub fn pair_to_mints(pair: &str) -> Option<(String, String)> {
    let (base_symbol, quote_symbol) = parse_pair(pair)?;
    let base_mint = symbol_to_mint(&base_symbol)?.to_string();
    let quote_mint = symbol_to_mint(&quote_symbol)?.to_string();
    Some((base_mint, quote_mint))
}

