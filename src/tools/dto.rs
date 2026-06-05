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
