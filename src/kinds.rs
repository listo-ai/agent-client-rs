//! Kind palette operations — hits `GET /api/v1/search?scope=kinds`.
//!
//! The agent exposes one generic search endpoint (see
//! `agent/docs/design/ANALYTICS.md` § "Global search"). This module is
//! an ergonomic wrapper: callers keep saying `client.kinds().list(...)`
//! and the wrapper routes the request through `/search` and unwraps
//! the `{ scope, hits, meta }` envelope.

use serde::Deserialize;

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::KindDto;

pub struct Kinds<'c> {
    http: &'c HttpClient,
    base: String,
}

/// Query options for [`Kinds::list_with`]. All fields are optional;
/// they compose on the server (`facet` + `filter` both apply).
#[derive(Debug, Default, Clone)]
pub struct ListKindsOptions<'a> {
    /// RSQL filter, e.g. `"org==com.listo"`.
    pub filter: Option<&'a str>,
    /// Comma-separated sort fields; prefix a field with `-` for descending.
    pub sort: Option<&'a str>,
    /// Concrete-param shortcut — facet filter (camelCase, `"isCompute"` …).
    pub facet: Option<&'a str>,
    /// Concrete-param shortcut — admits kinds the graph would accept
    /// under the given parent path.
    pub placeable_under: Option<&'a str>,
}

/// Envelope the server emits. Client wrappers unwrap `hits` so existing
/// call sites keep receiving `Vec<KindDto>`.
#[derive(Debug, Deserialize)]
struct SearchEnvelope<T> {
    #[allow(dead_code)]
    scope: String,
    hits: Vec<T>,
    #[allow(dead_code)]
    meta: SearchMeta,
}

#[derive(Debug, Deserialize)]
struct SearchMeta {
    #[allow(dead_code)]
    total: usize,
}

impl<'c> Kinds<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Back-compat shortcut: `list(facet, placeable_under)`. Prefer
    /// [`Self::list_with`] for new code.
    pub async fn list(
        &self,
        facet: Option<&str>,
        placeable_under: Option<&str>,
    ) -> Result<Vec<KindDto>, ClientError> {
        self.list_with(ListKindsOptions {
            facet,
            placeable_under,
            ..Default::default()
        })
        .await
    }

    /// Full-surface list. Empty options = "everything".
    pub async fn list_with(&self, opts: ListKindsOptions<'_>) -> Result<Vec<KindDto>, ClientError> {
        let mut parts: Vec<String> = vec!["scope=kinds".to_string()];
        if let Some(v) = opts.filter {
            parts.push(format!("filter={}", encode_value(v)));
        }
        if let Some(v) = opts.sort {
            parts.push(format!("sort={}", encode_value(v)));
        }
        if let Some(v) = opts.facet {
            parts.push(format!("facet={}", encode_value(v)));
        }
        if let Some(v) = opts.placeable_under {
            parts.push(format!("placeable_under={}", encode_value(v)));
        }
        let path = format!("{}/search?{}", self.base, parts.join("&"));
        let envelope: SearchEnvelope<KindDto> = self.http.get(&path).await?;
        Ok(envelope.hits)
    }
}

/// Minimal URL-encoder. RSQL operator tokens (`==`, `=prefix=`,
/// `=contains=`, …) use `=` as a literal delimiter, so leaving `=`
/// untouched is correct; we only encode characters the URL grammar
/// would misparse.
fn encode_value(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('#', "%23")
        .replace('&', "%26")
        .replace('?', "%3F")
}
