//! API interface module
//! Provides HTTP API interfaces for cross-chain history queries

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use hyperlane_core::H256;

use crate::db::{CrossChainHistory, PaginatedResult, ScraperDb};

/// API state, containing database connection
#[derive(Clone)]
pub struct ApiState {
    /// Database connection
    pub db: ScraperDb,
}

/// Request parameters for querying cross-chain history
#[derive(Debug, Deserialize)]
pub struct QueryHistoryParams {
    /// User address (hex format, with or without 0x prefix)
    pub address: String,
    /// Page number (starting from 1, default is 1)
    #[serde(default = "default_page")]
    pub page: u64,
    /// Page size (default is 20, maximum 100)
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}

/// API response structure
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Create API router
pub fn create_api_router(db: ScraperDb) -> Router {
    let state = ApiState { db };

    Router::new()
        .route("/api/v1/history", get(query_cross_chain_history))
        .with_state(Arc::new(state))
}

/// Query cross-chain history API
/// 
/// GET /api/v1/history?address=0x...&page=1&page_size=20
async fn query_cross_chain_history(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<QueryHistoryParams>,
) -> impl IntoResponse {
    info!(
        address = %params.address,
        page = params.page,
        page_size = params.page_size,
        "Querying cross-chain history"
    );

    // Validate and parse address
    let address = match parse_address(&params.address) {
        Ok(addr) => addr,
        Err(e) => {
            error!(error = %e, "Address parsing failed");
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<PaginatedResult<CrossChainHistory>>::error(format!(
                    "Invalid address format: {}",
                    e
                ))),
            );
        }
    };

    // Validate pagination parameters
    let page = if params.page == 0 { 1 } else { params.page };
    let page_size = params.page_size.min(100).max(1); // Limit page size between 1-100

    // Query database
    match state
        .db
        .query_cross_chain_history_by_address(&address, page, page_size)
        .await
    {
        Ok(result) => {
            info!(
                total = result.total,
                page = result.page,
                "Query successful"
            );
            (StatusCode::OK, Json(ApiResponse::success(result)))
        }
        Err(e) => {
            error!(error = %e, "Failed to query cross-chain history");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Query failed: {}", e))),
            )
        }
    }
}

/// Parse address string to H256
fn parse_address(address: &str) -> Result<H256, String> {
    // Remove 0x prefix (if present)
    let hex_str = address.strip_prefix("0x").unwrap_or(address);

    // Validate length (H256 requires 64 hex characters, but address might be 40 characters)
    if hex_str.len() != 64 && hex_str.len() != 40 {
        return Err(format!(
            "Invalid address length, expected 40 or 64 hex characters, got {}",
            hex_str.len()
        ));
    }

    // If it's a 40-character address, pad with zeros on the left to 64 characters
    let padded_hex = if hex_str.len() == 40 {
        format!("{:0>64}", hex_str)
    } else {
        hex_str.to_string()
    };

    // Parse to byte array
    let bytes = hex::decode(&padded_hex).map_err(|e| format!("Hex parsing failed: {}", e))?;

    // Convert to H256
    Ok(H256::from_slice(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_with_0x_prefix() {
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let result = parse_address(address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_without_0x_prefix() {
        let address = "1234567890abcdef1234567890abcdef12345678";
        let result = parse_address(address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_64_chars() {
        let address = "0x0000000000000000000000001234567890abcdef1234567890abcdef12345678";
        let result = parse_address(address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_invalid_length() {
        let address = "0x1234";
        let result = parse_address(address);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_address_invalid_hex() {
        let address = "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG";
        let result = parse_address(address);
        assert!(result.is_err());
    }
}
