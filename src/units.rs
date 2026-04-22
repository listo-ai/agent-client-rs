//! `GET /api/v1/units` — the public quantity / unit registry.
//!
//! Serves the shape clients need to render unit-picker UIs and drive
//! client-side previews when a user flips a preference.

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::UnitRegistryDto;

pub struct Units<'c> {
    http: &'c HttpClient,
    base: String,
}

impl<'c> Units<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Read the full quantity / unit registry. Safe to cache for the
    /// lifetime of the platform version — the registry is static per
    /// release.
    pub async fn get(&self) -> Result<UnitRegistryDto, ClientError> {
        let path = format!("{}/units", self.base);
        self.http.get::<UnitRegistryDto>(&path).await
    }
}
