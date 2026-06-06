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
    BillChargeArgs, BillChargeEntry, BillMemberAnalysisArgs, BillMemberAnalysisEntry,
    BillRevenueArgs, BillRevenueEntry, BusinessMetricsArgs, BusinessMetricsEntry, ListResponse,
    MemberAnalysisArgs, MemberAnalysisEntry, StationRevenueRankingArgs, StationRevenueRankingEntry,
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

    /// [Starcharger] period-bucketed charging ACTIVITY (not revenue).
    ///
    /// Session count, total charging seconds, and total kWh dispensed per period.
    /// Counts bills with status IN (12, 14, 18, 19) — INCLUDING cancellations and
    /// refunds — so it will NOT reconcile with `bill_revenue` (success-only).
    ///
    /// ## When to use
    ///
    /// Use `bill_charge` for "how much was the network USED" (energy/sessions);
    /// use `bill_revenue` for "how much was BILLED" (money). Their totals differ
    /// by design.
    ///
    /// ## Arguments
    ///
    /// * `start` / `end` / `freq` (default `week_sun`) / `seller_id` (omit for
    ///   network-wide).
    #[tool(
        description = "[Starcharger] period-bucketed charging ACTIVITY: session count, total charging seconds, total kWh dispensed. INCLUDES cancellations/refunds, so it does NOT reconcile with bill_revenue (success-only). Use for network usage, not money. Granularity via `freq` (default week_sun); optional `start`/`end`/`seller_id`."
    )]
    pub async fn bill_charge(
        &self,
        Parameters(args): Parameters<BillChargeArgs>,
    ) -> Result<Json<ListResponse<BillChargeEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, BillChargeEntry>("bill_charge", &args.window)
            .await?;
        Ok(Json(res))
    }

    /// [Starcharger] member acquisition & active-member counts per period.
    ///
    /// A TIME SERIES (one row per period): new sign-ups, cumulative member base,
    /// bills issued, and DISTINCT active members.
    ///
    /// ## When to use
    ///
    /// Use this for "how many members / how many distinct people charged". For a
    /// SEGMENT breakdown of orders (cohort × AC/DC × venue) use
    /// `bill_member_analysis` instead.
    ///
    /// ## Warning
    ///
    /// `charge_member` is a BILL COUNT, not distinct members — use
    /// `unique_charge_member` for the distinct-people metric. Periods with zero
    /// new members AND zero bills are dropped (not emitted as zero rows).
    ///
    /// ## Arguments
    ///
    /// * `start` / `end` / `freq` (default `week_sun`) / `seller_id`.
    #[tool(
        description = "[Starcharger] member acquisition + active-member counts per period (time series): new_members, total_members (cumulative), charge_member (BILL count), unique_charge_member (DISTINCT people). For segment breakdowns use bill_member_analysis. Granularity via `freq` (default week_sun); optional `start`/`end`/`seller_id`."
    )]
    pub async fn member_analysis(
        &self,
        Parameters(args): Parameters<MemberAnalysisArgs>,
    ) -> Result<Json<ListResponse<MemberAnalysisEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, MemberAnalysisEntry>("member_analysis", &args.window)
            .await?;
        Ok(Json(res))
    }

    /// [Starcharger] station & pile growth per period (network footprint).
    ///
    /// New stations / piles / running-stations that came online, plus the
    /// cumulative all-time installed totals, per period.
    ///
    /// ## Note
    ///
    /// `total_*` columns are cumulative across all time and never reset per year.
    /// `new_*_growth` ratios routinely exceed 1.0 (a value of `6.1` means a 7.1×
    /// jump). Ratios, not percentages.
    ///
    /// ## Arguments
    ///
    /// * `start` / `end` / `freq` (default `week_sun`) / `seller_id`.
    #[tool(
        description = "[Starcharger] station & pile growth per period: new_stations/new_piles/new_running_stations plus cumulative total_stations/total_piles/total_running_stations and POP growth ratios. Granularity via `freq` (default week_sun); optional `start`/`end`/`seller_id`."
    )]
    pub async fn business_metrics(
        &self,
        Parameters(args): Parameters<BusinessMetricsArgs>,
    ) -> Result<Json<ListResponse<BusinessMetricsEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, BusinessMetricsEntry>("business_metrics", &args.window)
            .await?;
        Ok(Json(res))
    }

    /// [Starcharger] success-only order counts cross-segmented by cohort × AC/DC × venue.
    ///
    /// A cross-tabulation: MANY rows per period — one per
    /// `(member_category × current_mode × construction)` cell — each with an
    /// `order_count`. Labels are Chinese (e.g. `2C會員`, `DC`, `超市`).
    ///
    /// ## Warning — response size
    ///
    /// The cross-product is LARGE. ALWAYS pass a TIGHT `start`/`end` (e.g. a
    /// single month) to bound the window; prefer a coarse `freq` (e.g. `month`).
    /// Do NOT query an unbounded or multi-quarter window.
    ///
    /// ## When to use
    ///
    /// Use this for "which member tiers / venues / AC vs DC drive orders". For a
    /// simple member-count time series use `member_analysis` instead.
    ///
    /// ## Arguments
    ///
    /// * `start` / `end` (keep tight!) / `freq` (default `week_sun`) / `seller_id`.
    #[tool(
        description = "[Starcharger] success-only ORDER COUNTS cross-segmented by member cohort × AC/DC mode × station venue (Chinese labels); MANY rows per period. Use for segment mix, NOT a member time series (that's member_analysis). LARGE output — ALWAYS pass a tight `start`/`end` (e.g. one month). Optional `seller_id`/`freq`."
    )]
    pub async fn bill_member_analysis(
        &self,
        Parameters(args): Parameters<BillMemberAnalysisArgs>,
    ) -> Result<Json<ListResponse<BillMemberAnalysisEntry>>, ErrorData> {
        let res = self
            .client
            .get_array_into_object::<_, BillMemberAnalysisEntry>(
                "bill_member_analysis",
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
