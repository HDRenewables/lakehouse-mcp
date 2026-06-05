//! Data structure definition
use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::query_param::PileDataQueryWindow;

/// One period-bucketed row of revenue metrics for the requested time window.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BillRevenueEntry {
    /// The period start in Asia/Taipei time zone.
    ///
    /// For example, `2025-01-01T00:00:00` for January 2025 at `freq=month`.
    pub label: NaiveDateTime,

    /// The unique identifier of seller of this row.
    ///
    /// Omitted from the response when the caller did not pass `seller_id`
    /// (network-wide query).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// Total charging hours in the period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_hour: Option<f64>,

    /// Total revenue in the period.
    ///
    /// Calculated with `Σ actual_balance`, success-only (`bill_status = 14`), currency units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue: Option<f64>,

    /// Discount ratio.
    ///
    /// Calculated with `(Σ original − Σ actual) / Σ actual`.
    /// A value of `0.25` means a 25 % discount on the actual amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_ratio: Option<f64>,

    /// Mean revenue per bill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_bill_value: Option<f64>,

    /// Revenue from AC piles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ac_revenue: Option<f64>,

    /// Revenue from DC piles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_dc_revenue: Option<f64>,

    /// **LEGACY** formula for AC analogue of `ac_revenue_per_kw`.
    ///
    /// ## Warning
    /// Calculated with `(Σ ac_balance / Σ ac_charge_power_kw) ×
    /// Σ ac_charge_duration_hr`. Dimensionally inconsistent, mixed units
    /// (currency/kWh × hours). Use `station_revenue_ranking.revenue_per_kw`
    /// for a clean figure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac_revenue_per_kw: Option<f64>,

    /// **LEGACY** formula for DC analogue of `dc_revenue_per_kw`.
    ///
    /// ## Warning
    /// Calculated with `(Σ dc_balance / Σ dc_charge_power_kw) ×
    /// Σ dc_charge_duration_hr`. Dimensionally inconsistent, mixed units
    /// (currency/kWh × hours). Use `station_revenue_ranking.revenue_per_kw`
    /// for a clean figure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_revenue_per_kw: Option<f64>,

    /// POP (period-over-period) ratio of `revenue`
    ///
    /// Calculated with `(cur − prev) / prev`.
    ///
    /// ## Note
    /// This value is ratio, not percentage. `1.2` means +120 % growth. First row of a
    /// partition reports 0. Division-by-zero collapses to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_growth: Option<f64>,

    /// POP ratio of `discount_ratio`.
    ///
    /// ## Note
    /// This value is ratio, not percentage. `1.2` means +120 % growth. First row of a
    /// partition reports 0. Division-by-zero collapses to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_ratio_growth: Option<f64>,

    /// POP ratio of `average_bill_value`.
    ///
    /// ## Note
    /// This value is ratio, not percentage. `1.2` means +120 % growth. First row of a
    /// partition reports 0. Division-by-zero collapses to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_bill_value_growth: Option<f64>,
}

/// Arguments for the `bill_revenue` tool — just the shared query window.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BillRevenueArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}

/// Envelope for general list-to-object queries.
///
/// The MCP spec requires a tool's `outputSchema` root to be an `object`, so list
/// responses are wrapped under `result`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListResponse<T> {
    /// The list of items returned by the query.
    pub result: Vec<T>,
}

/// One per-station row of the revenue ranking for a single period.
///
/// Rows arrive ordered by `total_revenue` descending (so with `limit=N` you get
/// the top-N stations). Each row is one `(station, period)` pair.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct StationRevenueRankingEntry {
    /// The period start in Asia/Taipei time zone (e.g. `2025-01-01T00:00:00`).
    pub label: NaiveDateTime,

    /// The seller (operator) owning this station. Always present.
    pub seller_id: i64,

    /// The station this row ranks. Always present.
    pub station_id: i64,

    /// Human-readable station name (from `t_station.name`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_name: Option<String>,

    /// Station capacity in kW — the **average** of daily measured capacity over
    /// the period (handles mid-period capacity changes), not a single snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_total_power: Option<f64>,

    /// Period revenue for this station (success-only). The ranking sort key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_revenue: Option<f64>,

    /// Total charging duration in the period, in hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_charging_duration_hour: Option<f64>,

    /// Energy dispensed in the period (kWh, across all bills).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_charge_kwh: Option<f64>,

    /// Energy dispensed by success-only bills (kWh).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_actual_charge_kwh: Option<f64>,

    /// Sum of station opening hours in the period (24 h/day fallback when no
    /// business-hour row exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_hours: Option<f64>,

    /// Capacity utilisation ratio — `Σ total_charge_kwh / Σ station_kwh`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_usage_rate: Option<f64>,

    /// **Clean** revenue-per-kW: `AVG(daily revenue) / AVG(daily station_power)`,
    /// units currency/kW. Typically single digits to low hundreds — prefer this
    /// over the dimensionally-inconsistent legacy `bill_revenue.*_revenue_per_kw`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_per_kw: Option<f64>,

    /// Period-over-period ratio of `total_revenue`, partitioned by
    /// `(station_id, seller_id)`.
    ///
    /// ## Note
    /// Ratio, not percentage (`1.2` = +120 %). Name says `wow` but it follows
    /// `freq`: MoM at `freq=month`, YoY at `freq=year`. First row of a partition
    /// reports 0; division-by-zero collapses to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_revenue_wow_growth: Option<f64>,

    /// POP ratio of `revenue_per_kw` (same caveats as `total_revenue_wow_growth`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_per_kw_wow_growth: Option<f64>,

    /// POP ratio of `total_charge_kwh` (same caveats as `total_revenue_wow_growth`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_charge_kwh_wow_growth: Option<f64>,
}

/// Arguments for the `station_revenue_ranking` tool — the shared query window.
///
/// Unlike most endpoints, `limit` IS honoured here (and `freq=day` is rejected).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StationRevenueRankingArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}

/// One period-bucketed row of charging ACTIVITY (not revenue).
///
/// Counts bills with status IN (12, 14, 18, 19) — i.e. INCLUDING cancellations
/// (12) and refunds (18, 19) — so totals will NOT reconcile with `bill_revenue`
/// (which is success-only). Use this for "how much was the network used",
/// `bill_revenue` for "how much was billed".
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BillChargeEntry {
    /// The period start in Asia/Taipei time zone (e.g. `2025-01-01T00:00:00`).
    pub label: NaiveDateTime,

    /// The seller (operator) of this row. Omitted for a network-wide query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// Number of bills/sessions in the period (includes refunds & cancellations).
    pub charge_count: i64,

    /// Total seconds plugged in (`Σ DATE_DIFF('second', start, end)`).
    pub charge_seconds: i64,

    /// Energy delivered in the period, in kWh (raw; includes refunded bills).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_power_kwh: Option<f64>,

    /// POP ratio of `charge_seconds`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_time_growth: Option<f64>,

    /// POP ratio of `charge_count`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_count_growth: Option<f64>,

    /// POP ratio of `charge_power_kwh`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_power_growth: Option<f64>,
}

/// Arguments for the `bill_charge` tool — the shared query window.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BillChargeArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}

/// One period-bucketed row of MEMBER counts (a time series, one row per period).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MemberAnalysisEntry {
    /// The period start in Asia/Taipei time zone.
    pub label: NaiveDateTime,

    /// The seller (operator) of this row. Omitted for a network-wide query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// Members whose `create_time` falls inside the period (new sign-ups).
    pub new_members: i64,

    /// Cumulative member count up to and including the period (only ever rises).
    pub total_members: i64,

    /// **Bill count** in the period — NOT distinct members. A member's repeated
    /// visits inflate this. For distinct people use `unique_charge_member`.
    pub charge_member: i64,

    /// `COUNT(DISTINCT member_id)` over bills in the period — distinct members
    /// who charged. This is the field for "how many different people charged".
    pub unique_charge_member: i64,

    /// POP ratio of `new_members`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_members_growth: Option<f64>,

    /// POP ratio of `total_members`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_members_growth: Option<f64>,

    /// POP ratio of `unique_charge_member`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_charge_member_growth: Option<f64>,
}

/// Arguments for the `member_analysis` tool — the shared query window.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemberAnalysisArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}

/// One period-bucketed row of network FOOTPRINT growth (stations & piles).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BusinessMetricsEntry {
    /// The period start in Asia/Taipei time zone.
    pub label: NaiveDateTime,

    /// The seller (operator) of this row. Omitted for a network-wide query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// Stations created in the period (`status IN (0, 1)`).
    pub new_stations: i64,
    /// Piles created in the period (orphaned piles excluded).
    pub new_piles: i64,
    /// Stations newly entering the running (`status = 1`) state in the period.
    pub new_running_stations: i64,

    /// Cumulative all-time station count (never resets per year).
    pub total_stations: i64,
    /// Cumulative all-time pile count.
    pub total_piles: i64,
    /// Cumulative all-time running-station count.
    pub total_running_stations: i64,

    /// POP ratio of `new_stations` (often > 1 for `new_*` series). Ratio, not %.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_stations_growth: Option<f64>,
    /// POP ratio of `new_piles`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_piles_growth: Option<f64>,
    /// POP ratio of `new_running_stations`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_running_stations_growth: Option<f64>,
    /// POP ratio of `total_stations`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_stations_growth: Option<f64>,
    /// POP ratio of `total_piles`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_piles_growth: Option<f64>,
    /// POP ratio of `total_running_stations`. Ratio, not percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_running_stations_growth: Option<f64>,
}

/// Arguments for the `business_metrics` tool — the shared query window.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BusinessMetricsArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}

/// One SEGMENT cell of success-only order counts.
///
/// The result is a cross-tabulation: there are MANY rows per period — one for
/// each `(member_category × current_mode × construction)` combination present.
/// This makes the response potentially very large; always query a tight window.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BillMemberAnalysisEntry {
    /// The period start in Asia/Taipei time zone.
    pub label: NaiveDateTime,

    /// The seller (operator) of this row. Omitted for a network-wide query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// Member loyalty cohort, Chinese label (e.g. `2C會員` retail, `2B會員`
    /// corporate, `Luxgen會員`, `Benz會員`, `Porsche會員`). `"Unknown"` for
    /// unrecognised codes.
    pub member_category: String,

    /// Pile charging mode — `"AC"`, `"DC"`, or `"Unknown"` (from the pile type,
    /// not the bill's charging_mode).
    pub current_mode: String,

    /// Station venue type, Chinese label (e.g. `超市` supermarket, `辦公大樓`
    /// office, `遊樂區` amusement area). `"Unknown"` for unrecognised codes.
    pub construction: String,

    /// `COUNT(bill)` in this cohort × mode × venue cell, success-only
    /// (`bill_status = 14`).
    pub order_count: i64,
}

/// Arguments for the `bill_member_analysis` tool — the shared query window.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BillMemberAnalysisArgs {
    #[serde(flatten)]
    pub window: PileDataQueryWindow,
}
