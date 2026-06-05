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
use crate::tools::dto::{
    BillRevenueArgs, BillRevenueEntry, ListResponse, StationRevenueRankingArgs,
    StationRevenueRankingEntry,
};

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
    /// * `start`: Optional. The start date (YYYY-MM-DD).
    /// * `end`: Optional. The end date (YYYY-MM-DD).
    /// * `freq`: Optional. The frequency of the data. Default is "week_sun".
    /// * `seller_id`: Pass a `seller_id` to scope to one seller (the response then carries a
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

    /// [Starcharger] per-station revenue ranking with the clean `revenue_per_kw`.
    ///
    /// One row per `(station, period)`, ordered by `total_revenue` DESC, with the
    /// dimensionally-clean `revenue_per_kw = AVG(daily revenue) / AVG(daily power)`
    /// plus energy, opening hours, and utilisation rate.
    ///
    /// ## Arguments
    ///
    /// * `limit`: **IMPORTANT: almost always pass this** (e.g. `limit=10` for a
    ///   top-10). Rows are sorted by `total_revenue` DESC and `limit` is applied as
    ///   a SQL `LIMIT`. DO NOT omit `limit` unless you explicitly need every
    ///   station in the network, otherwise the response is unbounded.
    /// * `freq`: Optional bucket granularity (default `week_sun`). To rank
    ///   stations across a whole window, choose a `freq` that yields ONE bucket
    ///   (e.g. `freq=quarter` for a quarter) so each station appears once.
    ///   **`freq=day` is rejected with an error.**
    /// * `start` / `end`: Optional window bounds (Asia/Taipei naive datetimes).
    /// * `seller_id`: Pass a `seller_id` to scope to one seller (the response then carries a
    /// `seller_id` column); omit it for a network-wide query.
    ///
    /// ## Note
    ///
    /// `*_wow_growth` is a ratio (not %) and follows `freq` (MoM at month, YoY at
    /// year). Use this endpoint's `revenue_per_kw`, not the legacy
    /// `bill_revenue.*_revenue_per_kw`.
    #[tool(
        description = "[Starcharger] per-station revenue ranking (clean revenue_per_kw), ordered by total_revenue DESC. ALWAYS pass `limit` (e.g. 10 for top-10) unless you explicitly want every station. `freq=day` is rejected; for a whole-window ranking use a single-bucket freq (e.g. quarter). Optional `start`/`end`/`seller_id`."
    )]
    pub async fn station_revenue_ranking(
        &self,
        Parameters(args): Parameters<StationRevenueRankingArgs>,
    ) -> Result<Json<ListResponse<StationRevenueRankingEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, StationRevenueRankingEntry>(
                "station_revenue_ranking",
                &args.window,
            )
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
