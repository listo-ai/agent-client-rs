//! Auth operations — `GET /api/v1/auth/whoami`.

use crate::error::ClientError;
use crate::http::HttpClient;
use crate::types::{EnrollRequest, EnrollResponse, SetupRequest, SetupResponse, WhoAmIDto};

pub struct Auth<'c> {
    http: &'c HttpClient,
    base: String,
}

impl<'c> Auth<'c> {
    pub(crate) fn new(http: &'c HttpClient, api_version: u32) -> Self {
        Self {
            http,
            base: format!("/api/v{api_version}"),
        }
    }

    /// Resolve the current auth context — who the agent thinks you are,
    /// which tenant, which scopes, which provider made the call.
    ///
    /// ```rust,no_run
    /// # async fn example(client: agent_client::AgentClient) -> Result<(), agent_client::ClientError> {
    /// let who = client.auth().whoami().await?;
    /// println!("signed in as {} ({})", who.actor_display, who.provider);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn whoami(&self) -> Result<WhoAmIDto, ClientError> {
        let path = format!("{}/auth/whoami", self.base);
        self.http.get::<WhoAmIDto>(&path).await
    }

    /// Run first-boot setup. Returns the initial bearer token which
    /// the caller should store (and pass to subsequent client builds
    /// via `AgentClient::with_token`). Fails with `409` if setup has
    /// already completed — see `SYSTEM-BOOTSTRAP.md`.
    ///
    /// ```rust,no_run
    /// # async fn example(client: agent_client::AgentClient) -> Result<(), agent_client::ClientError> {
    /// use agent_client::SetupRequest;
    /// let resp = client.auth().setup(&SetupRequest::Edge {}).await?;
    /// println!("token: {}", resp.token);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn setup(&self, req: &SetupRequest) -> Result<SetupResponse, ClientError> {
        let path = format!("{}/auth/setup", self.base);
        self.http.post::<SetupResponse, _>(&path, req).await
    }

    /// Enroll this edge with a cloud. Phase A: returns `501 Not
    /// Implemented` until the cloud-side handler + Zitadel provider
    /// land. The client surface is wired so callers do not have to
    /// change between Phase A and Phase B.
    pub async fn enroll(&self, req: &EnrollRequest) -> Result<EnrollResponse, ClientError> {
        let path = format!("{}/auth/enroll", self.base);
        self.http.post::<EnrollResponse, _>(&path, req).await
    }
}
