// Copyright 2026 Wayne Hong (h-alice) <contact@halice.art>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! HTTP relay around the upstream datacenter API.
//!
//! The MCP server does not implement any business logic, it forwards each
//! tool call to a single upstream `GET` endpoint.

use crate::appstate::AppState;
use crate::tools::dto::ListResponse;
use reqwest::StatusCode;
use rmcp::ErrorData;
use serde::{de::DeserializeOwned, Serialize};

/// HTTP client wrapper bound to one upstream base URL.
#[derive(Debug, Clone)]
pub struct ApiClient {
    state: AppState,
    http: reqwest::Client,
}

impl ApiClient {
    /// Build a client with the shared AppState.
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            http: reqwest::Client::new(),
        }
    }

    /// `GET {base}{path}?{params}`, decoding the response as a JSON array and wrapping it in `ListResponse`.
    ///
    /// # Errors
    /// - `invalid_params` when the upstream returns HTTP 400 (the only
    ///   client-fault status the pile_data API emits — e.g.
    ///   `station_revenue_ranking` with `freq=day`).
    /// - `internal_error` for any other non-success status, transport failure,
    ///   or a body that does not decode as `Vec<T>`.
    pub async fn get_array_into_object<P, T>(
        &self,
        slug: &str,
        params: &P,
    ) -> Result<ListResponse<T>, ErrorData>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        // Resolve slug to full path using the config
        let path = self
            .state
            .config
            .resolve_endpoint(slug)
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let base = self.state.config.api_base.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        let url = format!("{}/{}", base, path);
        tracing::debug!(%url, "relaying GET to upstream");

        // Invoke upstream API
        let resp = self
            .http
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("upstream request failed: {e}"), None)
            })?;

        // HTTP status check
        let status = resp.status();

        // 400: bad request
        if status == StatusCode::BAD_REQUEST {
            let body = resp.text().await.unwrap_or_default();
            return Err(ErrorData::invalid_params(
                format!("upstream rejected the request (400): {body}"),
                None,
            ));
        }

        // Any non-success status is treated as internal error
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ErrorData::internal_error(
                format!("upstream returned {status}: {body}"),
                None,
            ));
        }

        // Un-parsable JSON is treated as internal error
        let entries = resp.json::<Vec<T>>().await.map_err(|e| {
            ErrorData::internal_error(format!("failed to parse upstream JSON: {e}"), None)
        })?;

        Ok(ListResponse { result: entries })
    }
}
