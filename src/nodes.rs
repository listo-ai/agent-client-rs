//! Node operations.
//!
//! Listing goes through the generic search endpoint —
//! `GET /api/v1/search?scope=nodes` — and this wrapper unwraps the
//! envelope so callers keep receiving `NodeListResponse` /
//! `Vec<NodeSnapshot>`. Single-node reads and writes keep their
//! dedicated routes (`/api/v1/node`, `POST /api/v1/nodes`).

use serde::{Deserialize, Serialize};

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::{CreatedNode, NodeListResponse, NodeSchema, NodeSnapshot, PageMeta};

#[derive(Serialize)]
struct CreateNodeReq<'a> {
    parent: &'a str,
    kind: &'a str,
    name: &'a str,
}

pub struct Nodes<'c> {
    http: &'c HttpClient,
    base: String,
}

#[derive(Debug, Clone, Default)]
pub struct NodeListParams {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u64>,
    pub size: Option<u64>,
}

/// Envelope emitted by `/api/v1/search`. Pagination fields are optional
/// per-scope; the `nodes` scope always populates them.
#[derive(Debug, Deserialize)]
struct SearchEnvelope<T> {
    #[allow(dead_code)]
    scope: String,
    hits: Vec<T>,
    meta: SearchMeta,
}

#[derive(Debug, Deserialize)]
struct SearchMeta {
    total: u64,
    #[serde(default)]
    page: Option<u64>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    pages: Option<u64>,
}

impl<'c> Nodes<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// List every node in the graph (no pagination).
    pub async fn list(&self) -> Result<Vec<NodeSnapshot>, ClientError> {
        Ok(self.list_page(&NodeListParams::default()).await?.data)
    }

    /// List nodes via the generic search surface, preserving the
    /// existing `NodeListResponse` shape with pagination meta.
    pub async fn list_page(
        &self,
        params: &NodeListParams,
    ) -> Result<NodeListResponse, ClientError> {
        let mut query: Vec<(&str, String)> = vec![("scope", "nodes".to_string())];
        if let Some(filter) = &params.filter {
            query.push(("filter", filter.clone()));
        }
        if let Some(sort) = &params.sort {
            query.push(("sort", sort.clone()));
        }
        if let Some(page) = params.page {
            query.push(("page", page.to_string()));
        }
        if let Some(size) = params.size {
            query.push(("size", size.to_string()));
        }
        let envelope: SearchEnvelope<NodeSnapshot> = self
            .http
            .get_query(&format!("{}/search", self.base), &query)
            .await?;
        Ok(NodeListResponse {
            data: envelope.hits,
            meta: PageMeta {
                total: envelope.meta.total,
                page: envelope.meta.page.unwrap_or(1),
                size: envelope
                    .meta
                    .size
                    .unwrap_or(envelope.meta.total.max(1)),
                pages: envelope.meta.pages.unwrap_or(1),
            },
        })
    }

    /// Get a single node by its canonical path (e.g. `/station/floor1/ahu-5`).
    pub async fn get(&self, path: &str) -> Result<NodeSnapshot, ClientError> {
        let encoded = urlencoding_path(path);
        self.http
            .get::<NodeSnapshot>(&format!("{}/node?path={encoded}", self.base))
            .await
    }

    /// Get the kind-declared slot schemas for one node.
    pub async fn schema(
        &self,
        path: &str,
        include_internal: bool,
    ) -> Result<NodeSchema, ClientError> {
        let encoded = urlencoding_path(path);
        let url = format!(
            "{}/node/schema?path={encoded}&include_internal={include_internal}",
            self.base
        );
        self.http.get::<NodeSchema>(&url).await
    }

    /// Create a child node under `parent` with the given kind and name.
    pub async fn create(
        &self,
        parent: &str,
        kind: &str,
        name: &str,
    ) -> Result<CreatedNode, ClientError> {
        self.http
            .post(
                &format!("{}/nodes", self.base),
                &CreateNodeReq { parent, kind, name },
            )
            .await
    }

    /// Delete a node by path. Cascading behaviour depends on the node's kind.
    pub async fn delete(&self, path: &str) -> Result<(), ClientError> {
        let encoded = urlencoding_path(path);
        self.http
            .delete(&format!("{}/node?path={encoded}", self.base))
            .await
    }
}

fn urlencoding_path(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('#', "%23")
        .replace('&', "%26")
        .replace('?', "%3F")
}
