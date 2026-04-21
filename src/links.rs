//! Link operations — listing via `/api/v1/search?scope=links`, the
//! rest via dedicated `/api/v1/links/:id` routes.

use serde::{Deserialize, Serialize};

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::{CreatedLink, Link, LinkEndpointRef};

#[derive(Serialize)]
struct CreateLinkReq<'a> {
    source: &'a LinkEndpointRef,
    target: &'a LinkEndpointRef,
}

pub struct Links<'c> {
    http: &'c HttpClient,
    base: String,
}

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

impl<'c> Links<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// List all links in the graph via the generic search endpoint.
    pub async fn list(&self) -> Result<Vec<Link>, ClientError> {
        let envelope: SearchEnvelope<Link> = self
            .http
            .get(&format!("{}/search?scope=links", self.base))
            .await?;
        Ok(envelope.hits)
    }

    /// Create a link. Returns the new link ID.
    pub async fn create(
        &self,
        source: &LinkEndpointRef,
        target: &LinkEndpointRef,
    ) -> Result<String, ClientError> {
        let resp: CreatedLink = self
            .http
            .post(
                &format!("{}/links", self.base),
                &CreateLinkReq { source, target },
            )
            .await?;
        Ok(resp.id)
    }

    /// Remove a link by ID.
    pub async fn remove(&self, id: &str) -> Result<(), ClientError> {
        self.http
            .delete(&format!("{}/links/{id}", self.base))
            .await
    }
}
