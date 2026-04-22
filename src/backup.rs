//! Backup & restore — `POST /api/v1/backup/snapshot/{export,import}`.

use serde::{Deserialize, Serialize};

use crate::error::ClientError;
use crate::http::HttpClient;

/// Response from a snapshot export.
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotExportResult {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

/// Response from a snapshot import.
///
/// Phase 1: bundle is validated and staged; `status == "validated"`.
/// A future phase will add live DB-swap and return `status == "applied"`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotImportResult {
    pub status: String,
    pub agent_version: Option<String>,
    pub source_device_id: Option<String>,
    pub as_template: bool,
}

#[derive(Serialize)]
struct ExportReq<'a> {
    destination: &'a str,
}

#[derive(Serialize)]
struct ImportReq<'a> {
    bundle_path: &'a str,
    as_template: bool,
}

pub struct Backup<'c> {
    http: &'c HttpClient,
    base: String,
}

impl<'c> Backup<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}/backup"),
        }
    }

    /// Export a full snapshot to a local path on the agent host.
    pub async fn export_snapshot(
        &self,
        destination: &str,
    ) -> Result<SnapshotExportResult, ClientError> {
        let url = format!("{}/snapshot/export", self.base);
        self.http.post(&url, &ExportReq { destination }).await
    }

    /// Import (restore) a snapshot from a local path on the agent host.
    pub async fn import_snapshot(
        &self,
        bundle_path: &str,
        as_template: bool,
    ) -> Result<SnapshotImportResult, ClientError> {
        let url = format!("{}/snapshot/import", self.base);
        self.http
            .post(&url, &ImportReq { bundle_path, as_template })
            .await
    }

    /// Export a portability-filtered template bundle to a local path.
    pub async fn export_template(
        &self,
        destination: &str,
    ) -> Result<serde_json::Value, ClientError> {
        let url = format!("{}/template/export", self.base);
        self.http.post(&url, &ExportReq { destination }).await
    }

    /// Plan a template import — returns the conflict plan without applying.
    pub async fn plan_template_import(
        &self,
        bundle_path: &str,
        strategy: &str,
    ) -> Result<serde_json::Value, ClientError> {
        let url = format!("{}/template/import", self.base);
        self.http
            .post(
                &url,
                &serde_json::json!({ "bundle_path": bundle_path, "strategy": strategy }),
            )
            .await
    }
}
