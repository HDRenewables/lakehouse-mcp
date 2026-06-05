//! The MCP server module
//!
//! One `#[tool]` per upstream endpoint.
//!
//! `#[tool_router]` collects the annotated methods into a [`ToolRouter`].
//!
//! `#[tool_handler]` wires that router into [`ServerHandler`] so `tools/list`
//! and `tools/call` are dispatched automatically.
//!
//! Each tool's `///` doc-comment (and the per-field docs on the output struct)
//! is what the LLM sees in `tools/list` / `outputSchema`, make MCP server self-documenting.

use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData, Json, ServerHandler,
};

use crate::client::ApiClient;
use crate::conventions;
use crate::tools::dto::{BillRevenueArgs, BillRevenueEntry, ListResponse};

/// MCP server over the datacenter APIs. Holds the upstream HTTP client
/// and the macro-generated tool router.
#[derive(Clone)]
pub struct EomcServer {
    /// The upstream HTTP client.
    client: Arc<ApiClient>,
    /// The macro-generated tool router.
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl EomcServer {
    /// Build the server around an upstream API client.
    pub fn new(client: Arc<ApiClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    /// [Starcharger] period-bucketed revenue totals (success-only)
    ///
    /// AC/DC split, discount ratio, legacy per-kW, and period-over-period growth.
    ///
    /// Returns one row per period (granularity set by `freq`, default
    /// `week_sun`). Revenue is success-only (`bill_status = 14`); for activity
    /// counts that include refunds use `bill_charge`.
    ///
    /// ## Arguments
    ///
    /// * `start` - Optional. The start date (YYYY-MM-DD).
    /// * `end` - Optional. The end date (YYYY-MM-DD).
    /// * `freq` - Optional. The frequency of the data. Default is "week_sun".
    /// * `seller_id` - Pass a `seller_id` to scope to one operator (the response then carries a
    /// `seller_id` column); omit it for a network-wide query.
    ///
    /// ## Warning
    ///
    /// The `*_revenue_per_kw` columns use a LEGACY, dimensionally-inconsistent formula
    /// prefer `station_revenue_ranking.revenue_per_kw` for a clean ratio.
    #[tool(
        description = "[Starcharger] period-bucketed revenue totals (success-only): total revenue, AC/DC split, discount ratio, legacy per-kW, and period-over-period growth ratios. Granularity via `freq` (default week_sun); optional `start`/`end`/`seller_id`."
    )]
    pub async fn bill_revenue(
        &self,
        Parameters(args): Parameters<BillRevenueArgs>,
    ) -> Result<Json<ListResponse<BillRevenueEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, BillRevenueEntry>("bill_revenue", &args.window)
            .await?;
        Ok(Json(res))
    }
}

/// Server handler that wires the tool router into the MCP server.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for EomcServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(conventions::INSTRUCTIONS.to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
