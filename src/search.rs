//! Generic search API — hits `GET /api/v1/search?scope=<id>` with any
//! scope id and returns the raw envelope.
//!
//! The per-scope wrappers (`client.kinds()`, `client.flows()`, etc.)
//! stay the ergonomic surface for typed results. This module is for
//! callers that want to dispatch across scopes dynamically — the CLI's
//! `agent find` verb, the MCP `search` tool, and Studio's fleet proxy.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::error::ClientError;
use crate::http::HttpClient;

pub struct Search<'c> {
    http: &'c HttpClient,
    base: String,
}

/// Per-call parameters. All fields optional; `scope` is the only one
/// the server requires.
#[derive(Debug, Default, Clone)]
pub struct SearchParams<'a> {
    /// Scope id — `"kinds"`, `"nodes"`, `"blocks"`, `"links"`, `"flows"`.
    pub scope: &'a str,
    /// RSQL filter — field set is scope-specific.
    pub filter: Option<&'a str>,
    /// Comma-separated sort fields; `-field` for descending.
    pub sort: Option<&'a str>,
    /// Concrete-param shortcut (scope-specific): `kinds` uses `facet`.
    pub facet: Option<&'a str>,
    /// Concrete-param shortcut (scope-specific): `kinds` uses `placeable_under`.
    pub placeable_under: Option<&'a str>,
    /// Pagination (scopes that paginate honour these; others ignore).
    pub page: Option<u64>,
    pub size: Option<u64>,
}

/// Wire envelope — scope id, hit rows (type-erased as JSON), meta.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchEnvelope {
    pub scope: String,
    pub hits: Vec<JsonValue>,
    pub meta: SearchMeta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchMeta {
    pub total: u64,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub pages: Option<u64>,
}

impl<'c> Search<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Execute a search with the given params.
    pub async fn query(&self, params: SearchParams<'_>) -> Result<SearchEnvelope, ClientError> {
        let mut parts: Vec<String> = vec![format!("scope={}", encode(params.scope))];
        if let Some(v) = params.filter {
            parts.push(format!("filter={}", encode(v)));
        }
        if let Some(v) = params.sort {
            parts.push(format!("sort={}", encode(v)));
        }
        if let Some(v) = params.facet {
            parts.push(format!("facet={}", encode(v)));
        }
        if let Some(v) = params.placeable_under {
            parts.push(format!("placeable_under={}", encode(v)));
        }
        if let Some(v) = params.page {
            parts.push(format!("page={v}"));
        }
        if let Some(v) = params.size {
            parts.push(format!("size={v}"));
        }
        let path = format!("{}/search?{}", self.base, parts.join("&"));
        self.http.get::<SearchEnvelope>(&path).await
    }
}

fn encode(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('#', "%23")
        .replace('&', "%26")
        .replace('?', "%3F")
}
