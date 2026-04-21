//! Kind operations — `GET /api/v1/kinds`.
//!
//! Backed by the RSQL query surface documented in
//! `docs/design/QUERY-LANG.md`: `filter` + `sort` expressions over
//! `id` / `org` / `display_name` / `facets` / `placement_class`, plus
//! the `facet` and `placeable_under` concrete-param shortcuts.

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

impl<'c> Kinds<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Back-compat shortcut: `list(facet, placeable_under)`. Prefer
    /// [`Self::list_with`] for new code — it accepts the full query
    /// surface in one options struct.
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
        let mut parts: Vec<String> = Vec::new();
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
        let path = if parts.is_empty() {
            format!("{}/kinds", self.base)
        } else {
            format!("{}/kinds?{}", self.base, parts.join("&"))
        };
        self.http.get::<Vec<KindDto>>(&path).await
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
