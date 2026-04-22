//! User + org preferences — matches the server endpoints in
//! [`agent/crates/transport-rest/src/preferences.rs`]:
//!
//! - `GET  /api/v1/me/preferences?org=<id>`      → resolved view
//! - `PATCH /api/v1/me/preferences?org=<id>`     → update user layer
//! - `GET  /api/v1/orgs/{id}/preferences`        → org layer (admin)
//! - `PATCH /api/v1/orgs/{id}/preferences`       → update org layer (admin)
//!
//! A `PATCH` body is sparse. Use the helpers on [`PreferencesPatch`]:
//!
//! ```rust,no_run
//! use agent_client::{AgentClient, PreferencesPatch};
//!
//! # async fn demo(client: AgentClient) -> Result<(), agent_client::ClientError> {
//! let patch = PreferencesPatch::default()
//!     .set("theme", "dark")           // explicit value
//!     .clear("temperature_unit");     // revert to inherit-from-org
//! client.preferences().patch_mine(None, &patch).await?;
//! # Ok(()) }
//! ```
//!
//! Semantics of the three states for a PATCH field:
//!
//! - **Absent**  (field not set in the patch)  → server leaves stored value unchanged.
//! - **Clear**   (`set` → JSON `null`)         → server clears the user layer and inherits from org / defaults.
//! - **Set(v)**  (value provided)              → server writes `v` to the user layer.

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::{OrgPreferences, PreferencesPatch, ResolvedPreferences};

pub struct Preferences<'c> {
    http: &'c HttpClient,
    base: String,
}

impl<'c> Preferences<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Resolved preferences for the current caller, scoped to `org` (or
    /// the active tenant in the auth context when `org` is `None`).
    pub async fn get_mine(&self, org: Option<&str>) -> Result<ResolvedPreferences, ClientError> {
        let path = match org {
            Some(o) => format!("{}/me/preferences?org={}", self.base, urlencoded(o)),
            None => format!("{}/me/preferences", self.base),
        };
        self.http.get::<ResolvedPreferences>(&path).await
    }

    /// Patch the user-per-org layer. Returns the fresh resolved view so
    /// callers don't need a follow-up GET.
    pub async fn patch_mine(
        &self,
        org: Option<&str>,
        patch: &PreferencesPatch,
    ) -> Result<ResolvedPreferences, ClientError> {
        let path = match org {
            Some(o) => format!("{}/me/preferences?org={}", self.base, urlencoded(o)),
            None => format!("{}/me/preferences", self.base),
        };
        self.http
            .patch::<ResolvedPreferences, _>(&path, patch)
            .await
    }

    /// Read the org-layer row. Admin-only (`Scope::Admin`); non-admin
    /// callers get `403 Forbidden` from the server.
    pub async fn get_org(&self, org_id: &str) -> Result<OrgPreferences, ClientError> {
        let path = format!("{}/orgs/{}/preferences", self.base, urlencoded(org_id));
        self.http.get::<OrgPreferences>(&path).await
    }

    /// Patch the org-layer row. Admin-only. Returns the updated row.
    pub async fn patch_org(
        &self,
        org_id: &str,
        patch: &PreferencesPatch,
    ) -> Result<OrgPreferences, ClientError> {
        let path = format!("{}/orgs/{}/preferences", self.base, urlencoded(org_id));
        self.http.patch::<OrgPreferences, _>(&path, patch).await
    }
}

/// Minimal percent-encoder for path-segment and query-value use.
/// Avoids adding a `url` / `percent-encoding` dependency for one
/// function. Encodes everything outside the unreserved set per
/// RFC 3986.
fn urlencoded(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
