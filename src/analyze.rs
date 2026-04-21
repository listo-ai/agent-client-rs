//! Ad-hoc analytical compute — `POST /api/v1/analyze`.
//!
//! The wire shape is locked now; the server returns
//! [`ClientError::Http`] with a 503 body `{code:"analytics_unavailable",
//! …}` until the analytics-engine sidecar is deployed. Callers should
//! surface that as "analytics not available on this agent" rather than
//! a hard error.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::error::ClientError;
use crate::http::HttpClient;

pub struct Analyze<'c> {
    http: &'c HttpClient,
    base: String,
}

/// Request payload for [`Analyze::run`].
#[derive(Debug, Default, Clone, Serialize)]
pub struct AnalyzeRequest {
    /// Named Dataset references or inline Dataset bodies keyed by the
    /// table name the SQL stage references.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub inputs: serde_json::Map<String, JsonValue>,
    /// DataFusion SQL. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
    /// Rhai script. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rhai: Option<String>,
    /// Post-SQL row cap. `None` = server default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_cap: Option<u64>,
    /// Per-call timeout. `None` = server default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// Response shape. Matches the eventual analytics-engine sidecar
/// payload 1:1 so the shim is transparent when the sidecar lands.
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeResponse {
    pub rows: Vec<JsonValue>,
    pub meta: AnalyzeMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeMeta {
    pub rows_in: u64,
    pub rows_out: u64,
    pub duration_ms: u64,
    pub dry_run: bool,
}

impl<'c> Analyze<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Execute an ad-hoc rule. Returns the raw rows + metadata; no
    /// intents are ever emitted from this endpoint (it's `dry_run`
    /// by construction).
    pub async fn run(&self, req: &AnalyzeRequest) -> Result<AnalyzeResponse, ClientError> {
        self.http
            .post(&format!("{}/analyze", self.base), req)
            .await
    }
}
